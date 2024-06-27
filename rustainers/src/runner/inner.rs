use std::borrow::Cow;
use std::collections::HashMap;
use std::env;
use std::fmt::{Debug, Display};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::Path;
use std::time::Duration;

use async_trait::async_trait;
use indexmap::IndexMap;
use serde::de::DeserializeOwned;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tracing::{debug, info, trace, warn};

use crate::cmd::Cmd;
use crate::io::StdIoKind;
use crate::{
    ContainerHealth, ContainerId, ContainerProcess, ContainerState, ContainerStatus, ExposedPort,
    HealthCheck, HostContainer, Ip, IpamNetworkConfig, Network, NetworkDetails, NetworkInfo, Port,
    RunnableContainer, Volume, WaitStrategy,
};

use super::{ContainerError, RunOption};

#[async_trait]
pub(crate) trait InnerRunner: Display + Debug + Send + Sync {
    fn command(&self) -> Cmd<'static>;

    #[tracing::instrument(level = "debug", skip(self), fields(runner = %self))]
    async fn ps(&self, name: &str) -> Result<Option<ContainerProcess>, ContainerError> {
        let mut cmd = self.command();
        cmd.push_args([
            "ps",
            "--all",
            "--no-trunc",
            "--filter",
            &format!("name={name}"),
            "--format={{json .}}",
        ]);

        let containers = cmd.json_stream::<ContainerProcess>().await?;
        let result = containers.into_iter().find(|it| it.names.contains(name));
        Ok(result)
    }

    #[tracing::instrument(level = "debug", skip(self), fields(runner = %self))]
    async fn create_network(&self, name: &str) -> Result<(), ContainerError> {
        let mut cmd = self.command();
        cmd.push_args(["network", "create", name]);
        cmd.status().await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self), fields(runner = %self))]
    async fn create_volume(&self, name: &str) -> Result<(), ContainerError> {
        let mut cmd = self.command();
        cmd.push_args(["volume", "create", name]);
        cmd.status().await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, option), fields(runner = %self))]
    async fn create_and_start(
        &self,
        option: CreateAndStartOption<'_>,
    ) -> Result<ContainerId, ContainerError> {
        let mut cmd = self.command();
        cmd.push_args(["run", "--detach"]);

        // Remove
        if option.remove {
            cmd.push_arg("--rm");
        }

        // Name
        if let Some(name) = option.name {
            cmd.push_args(["--name", name]);
        }

        // Env. vars.
        for (key, value) in option.env {
            let env_var = format!("{key}={value}");
            cmd.push_args(["--env", &env_var]);
        }

        // Published ports
        for port_mapping in option.ports {
            let publish = port_mapping.to_publish().await;
            cmd.push_args(["--publish", &publish]);
        }

        // Health check args.
        if let Some(hc) = &option.health_check {
            cmd.push_args(hc.to_vec());
        }

        // Network
        let network = option.network.cmd_arg();
        cmd.push_arg(network.as_ref());

        // Volumes
        for volume in option.volumes {
            cmd.push_arg("--mount");
            cmd.push_arg(&volume.mount_arg()?);
        }

        // Entrypoint
        if let Some(entrypoint) = option.entrypoint {
            cmd.push_args(["--entrypoint", entrypoint]);
        }

        // Descriptor (name:tag or other alternatives)
        let descriptor = &option.descriptor;
        cmd.push_arg(descriptor);

        // Command
        let command_args = option.command;
        cmd.push_args(command_args);

        // Run
        info!(image = %descriptor, "🚀 Launching container");
        let stdout = cmd.result().await?;
        let id = stdout.trim().parse::<ContainerId>()?;

        Ok(id)
    }

    #[tracing::instrument(level = "debug", skip(self, id), fields(runner = %self, id = %id))]
    async fn inspect<R>(&self, id: ContainerId, json_path: &str) -> Result<R, ContainerError>
    where
        R: DeserializeOwned + Default + Debug,
    {
        let mut cmd = self.command();
        cmd.push_args(["inspect", &format!("--format={{{{json {json_path}}}}}")]);
        cmd.push_arg(id);
        let result = cmd.json::<R>().await?;
        Ok(result)
    }

    #[tracing::instrument(level = "debug", skip(self, id), fields(runner = %self, id = %id))]
    async fn port(&self, id: ContainerId, container_port: Port) -> Result<Port, ContainerError> {
        let mut cmd = self.command();
        cmd.push_arg("port");
        cmd.push_arg(id);
        cmd.push_arg(container_port);
        let output = cmd.result().await?;
        parse_port(&output).ok_or_else(|| {
            warn!( %id, %container_port, "Bound port not found\n{cmd}\noutput: '{output}'");
            ContainerError::PortNotFound { id, container_port }
        })
    }

    #[tracing::instrument(level = "debug", skip(self, id), fields(runner = %self, id = %id))]
    async fn start(&self, id: ContainerId) -> Result<(), ContainerError> {
        let mut cmd = self.command();
        cmd.push_arg("start");
        cmd.push_arg(id);
        let status = cmd.status().await?;
        if status.success() {
            info!(%id, "▶️ Container started");
            Ok(())
        } else {
            warn!(%id, ?status, "⚠️ Fail to start container");
            Err(ContainerError::ContainerCannotBeStarted(id))
        }
    }

    #[tracing::instrument(level = "debug", skip(self, id), fields(runner = %self, id = %id))]
    async fn unpause(&self, id: ContainerId) -> Result<(), ContainerError> {
        let mut cmd = self.command();
        cmd.push_arg("unpause");
        cmd.push_arg(id);
        let status = cmd.status().await?;
        if status.success() {
            info!(%id, "⏯ Container resumed");
            Ok(())
        } else {
            warn!(%id, ?status, "⚠️ Fail to resume container");
            Err(ContainerError::ContainerCannotBeResumed(id))
        }
    }

    async fn full_status(&self, id: ContainerId) -> Result<ContainerState, ContainerError> {
        self.inspect(id, ".State").await
    }

    async fn network_ip(
        &self,
        id: ContainerId,
        network: &str,
    ) -> Result<NetworkDetails, ContainerError> {
        let mut networks = self.inspect_networks(id).await?;
        if let Some((_, network)) = networks.remove_entry(network) {
            Ok(network)
        } else {
            Err(ContainerError::NoNetwork)
        }
    }

    #[tracing::instrument(level = "debug", skip(self), fields(runner = %self))]
    async fn inspect_networks(
        &self,
        id: ContainerId,
    ) -> Result<HashMap<String, NetworkDetails>, ContainerError> {
        let path = ".NetworkSettings.Networks".to_string();
        self.inspect(id, &path).await
    }

    #[tracing::instrument(level = "debug", skip(self), fields(runner = %self))]
    async fn inspect_host_containers(
        &self,
        id: ContainerId,
    ) -> Result<HashMap<ContainerId, HostContainer>, ContainerError> {
        let path = ".Containers".to_string();
        self.inspect(id, &path).await
    }

    #[tracing::instrument(level = "debug", skip(self), fields(runner = %self))]
    async fn list_networks(&self, name: &str) -> Result<Vec<NetworkInfo>, ContainerError> {
        let mut cmd = self.command();
        cmd.push_args([
            "network",
            "ls",
            "--no-trunc",
            "--filter",
            &format!("type={name}"),
            "--format={{json .}}",
        ]);
        let result = cmd.json_stream::<NetworkInfo>().await?;
        Ok(result)
    }

    #[tracing::instrument(level = "debug", skip(self, id), fields(runner = %self, id = %id))]
    async fn wait_ready(
        &self,
        id: ContainerId,
        wait_condition: &WaitStrategy,
        interval: Duration, // TODO could have a more flexible type
    ) -> Result<(), ContainerError> {
        if let WaitStrategy::LogMatch { io, matcher } = wait_condition {
            let mut rx = self.watch_logs(id, *io).await?;
            while let Some(line) = rx.recv().await {
                trace!("Log: {line}");
                if matcher.matches(&line) {
                    return Ok(());
                }
            }
            return Err(ContainerError::WaitConditionUnreachable(
                id,
                wait_condition.clone(),
            ));
        }

        // Other cases
        loop {
            match wait_condition {
                WaitStrategy::HealthCheck | WaitStrategy::CustomHealthCheck(_) => {
                    if self.check_healthy(id).await? {
                        info!(%id, "💚 healthy");
                        break;
                    }
                }
                WaitStrategy::State(state) => {
                    if self.check_for_state(id, *state).await? {
                        info!(%id, "💚 state {state} reached");
                        break;
                    }
                }
                WaitStrategy::HttpSuccess {
                    https,
                    path,
                    container_port,
                } => {
                    let Ok(host_port) = self.port(id, *container_port).await else {
                        info!(%container_port,"Port not bind, will retry later");
                        continue;
                    };
                    let scheme = if *https { "https" } else { "http" };
                    let url = format!(
                        "{scheme}://127.0.0.1:{host_port}/{}",
                        path.trim_start_matches('/')
                    );
                    let Ok(response) = reqwest::get(&url).await else {
                        warn!(%url,"Fail to get the URL, will retry later");
                        continue;
                    };
                    let status = response.status();
                    if status.is_success() {
                        info!(%id, %status, "💚 {url} successful");
                        break;
                    }
                    debug!(%status, "{url} not yet ready, will retry later");
                }
                WaitStrategy::ScanPort {
                    container_port,
                    timeout,
                } => {
                    let Ok(host_port) = self.port(id, *container_port).await else {
                        info!(%container_port,"Port not bind, will retry later");
                        continue;
                    };
                    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), host_port.0);

                    let scan =
                        tokio::time::timeout(
                            *timeout,
                            async move { TcpStream::connect(addr).await },
                        )
                        .await;
                    if let Ok(Ok(_)) = scan {
                        info!(%id, %container_port, %host_port, "💚 port {container_port} available");
                        break;
                    }
                    debug!(%id, %container_port, %host_port, "Port {container_port} not yet available, will retry later");
                }
                WaitStrategy::None => {
                    break;
                }
                WaitStrategy::LogMatch { .. } => {
                    unreachable!("This case is handled outside the loop")
                }
            }

            tokio::time::sleep(interval).await;
        }

        Ok(())
    }

    fn get_docker_host(&self) -> Option<String> {
        env::var("DOCKER_HOST").ok()
    }

    #[tracing::instrument(skip(self),fields(runner = %self))]
    async fn find_host_network(&self) -> Result<Option<Network>, ContainerError> {
        // If we're docker in docker running on a custom network, we need to inherit the
        // network settings, so we can access the resulting container.
        let docker_host = self.host().await?;
        let custom_networks = self.list_networks("custom").await?;
        let mut networks = vec![];
        for network in custom_networks {
            let path = ".IPAM.Config".to_string();
            let network_configs: Vec<IpamNetworkConfig> = self.inspect(network.id, &path).await?;
            networks.extend(
                network_configs
                    .into_iter()
                    .filter(|x| {
                        if let Some(subnet) = x.subnet {
                            subnet.contains(IpAddr::V4(docker_host.0))
                        } else {
                            false
                        }
                    })
                    .map(|_| Network::Custom(network.name.clone()))
                    .collect::<Vec<Network>>(),
            );
        }
        if let [network] = &networks[..] {
            Ok(Some(network.clone()))
        } else {
            Ok(None)
        }
    }

    #[tracing::instrument(skip(self),fields(runner = %self))]
    async fn host(&self) -> Result<Ip, ContainerError> {
        if self.is_inside_container() {
            self.default_gateway_ip().await
        } else {
            Ok(Ip(Ipv4Addr::LOCALHOST))
        }
    }

    #[tracing::instrument(skip(self), fields(runner = %self))]
    async fn default_gateway_ip(&self) -> Result<Ip, ContainerError> {
        let hostname = env::var("HOSTNAME")?;
        let host_id = hostname.parse::<ContainerId>()?;
        let networks = self.inspect_networks(host_id).await?;
        let mut gateways = vec![];
        for (_, network) in networks {
            // let network_id = name.parse::<ContainerId>()?;
            if let Some(network_id) = network.id {
                let containers = self.inspect_host_containers(network_id).await?;
                // Due to short id vs long id
                let container_ids = containers
                    .keys()
                    .map(std::string::ToString::to_string)
                    .collect::<Vec<String>>();
                if container_ids.contains(&hostname) {
                    if let Some(gateway) = network.gateway {
                        gateways.push(gateway);
                    }
                }
            }
        }
        if let [gateway] = gateways[..] {
            Ok(gateway)
        } else {
            Err(ContainerError::NoGateway)
        }
    }

    fn is_inside_container(&self) -> bool {
        Path::new("/.dockerenv").exists()
    }

    async fn watch_logs(
        &self,
        id: ContainerId,
        io: StdIoKind,
    ) -> Result<mpsc::Receiver<String>, ContainerError> {
        let mut cmd = self.command();
        cmd.push_args(["logs", "--follow"]);
        cmd.push_arg(id);

        let (tx, rx) = mpsc::channel(256);
        tokio::spawn(async move { cmd.watch_io(io, tx).await });

        Ok(rx)
    }

    async fn check_healthy(&self, id: ContainerId) -> Result<bool, ContainerError> {
        let state = self.full_status(id).await?;
        if !matches!(
            state.status,
            ContainerStatus::Restarting | ContainerStatus::Running
        ) {
            warn!(%id, ?state, "✋ The container not seems to run");
            let state = format!("{:?}", state.status);
            return Err(ContainerError::InvalidContainerState(id, state));
        }
        match state.health.status {
            ContainerHealth::Healthy => Ok(true),
            ContainerHealth::Unhealthy => {
                info!(%id, "🚨 unhealthy");
                Err(ContainerError::UnhealthyContainer(id))
            }
            ContainerHealth::Starting => {
                // TODO use another way to display logs (like tokio channel)
                if let Some(last_log) = state.health.log.unwrap_or_default().last() {
                    trace!(%id, "Last health check log\n\t{}", last_log);
                }
                Ok(false)
            }
            ContainerHealth::Unknown => {
                warn!(%id, ?state, "🩺 The container does not have health check");
                Err(ContainerError::UnknownContainerHealth(id))
            }
        }
    }

    async fn check_for_state(
        &self,
        id: ContainerId,
        state: ContainerStatus,
    ) -> Result<bool, ContainerError> {
        let status = self.full_status(id).await?;
        Ok(status.status == state)
    }

    #[tracing::instrument(level = "debug", skip(self, id), fields(runner = %self, id = %id))]
    async fn rm(&self, id: ContainerId) -> Result<(), ContainerError> {
        let mut cmd = self.command();
        cmd.push_arg("rm");
        cmd.push_arg(id);
        let status = cmd.status().await?;

        if status.success() {
            info!(%id, "🧹 Container removed");
            Ok(())
        } else {
            warn!(%id, ?status, "⚠️ Fail to remove container");
            Err(ContainerError::ContainerCannotBeRemoved(id))
        }
    }

    #[tracing::instrument(skip(self, image), fields(runner = %self, image = %image))]
    async fn start_container(
        &self,
        image: &mut RunnableContainer,
        options: RunOption,
    ) -> Result<ContainerId, ContainerError> {
        // Container name
        let name = options.name.as_deref();
        let container_name = image.container_name.as_deref().or(name);
        let container = if let Some(name) = container_name {
            self.ps(name).await?.map(|it| (it.state, it.id))
        } else {
            None
        };

        let id = match container {
            // Nothing to do for the container
            Some((ContainerStatus::Restarting | ContainerStatus::Running, id)) => id,
            // Need to unpause the container
            Some((ContainerStatus::Paused, id)) => {
                self.unpause(id).await?;
                id
            }
            // Need to start the container
            Some((
                ContainerStatus::Created | ContainerStatus::Exited | ContainerStatus::Stopped,
                id,
            )) => {
                self.start(id).await?;
                id
            }
            // Need cleanup before restarting the container
            Some((ContainerStatus::Dead, id)) => {
                self.rm(id).await?;
                self.create_and_start(CreateAndStartOption::new(image, &options))
                    .await?
            }
            // Need to create and start the container
            Some((ContainerStatus::Unknown | ContainerStatus::Removing, _)) => {
                self.create_and_start(CreateAndStartOption::new(image, &options))
                    .await?
            }
            None => {
                // If the user has specified a network, we'll assume the user knows best
                let options = if options.network.is_none() & self.get_docker_host().is_none() {
                    // Otherwise we'll try to find the docker host for dind usage.
                    let host_network = self.find_host_network().await?;
                    // Not ideal to clone the options to modify it
                    let mut options = options.clone();
                    options.network = host_network;
                    options
                } else {
                    options.clone()
                };
                self.create_and_start(CreateAndStartOption::new(image, &options))
                    .await?
            }
        };

        // Wait
        // TODO maybe set a timeout
        self.wait_ready(id, &image.wait_strategy, options.wait_interval)
            .await?;

        // Port Mapping
        for port_mapping in &mut image.port_mappings {
            let host_port = self.port(id, port_mapping.container_port).await?;
            port_mapping.bind_port(host_port).await;
        }

        Ok(id)
    }

    #[tracing::instrument(skip(self, id), fields(runner = %self, id = %id))]
    async fn exec(
        &self,
        id: ContainerId,
        exec_command: Vec<String>,
    ) -> Result<String, ContainerError> {
        let mut cmd = self.command();
        cmd.push_arg("exec");
        cmd.push_arg(id);
        cmd.push_args(exec_command);

        let stdout = cmd.result().await?;
        info!(%id, "🐚 Executed\n{stdout}",);

        Ok(stdout)
    }

    #[tracing::instrument(skip(self, id), fields(runner = %self, id = %id))]
    fn stop(&self, id: ContainerId) -> Result<(), ContainerError> {
        let mut cmd = self.command();
        cmd.push_arg("stop");
        cmd.push_arg(id);
        let status = cmd.status_blocking()?;
        if status.success() {
            info!(%id, "🛑 Container stopped");
        } else {
            warn!(%id, ?status, "⚠️ Fail to stop container");
        }
        Ok(())
    }
}

fn parse_port(str: &str) -> Option<Port> {
    str.lines()
        .filter_map(|it| it.parse::<SocketAddr>().ok())
        .map(|it| Port(it.port()))
        .next()
}

pub(crate) struct CreateAndStartOption<'a> {
    descriptor: String,
    health_check: Option<&'a HealthCheck>,
    ports: &'a [ExposedPort],
    remove: bool,
    name: Option<&'a str>,
    network: Cow<'a, Network>,
    volumes: &'a [Volume],
    env: IndexMap<&'a str, &'a str>,
    command: &'a [String],
    entrypoint: Option<&'a str>,
}

impl<'a> CreateAndStartOption<'a> {
    pub(super) fn new<'b: 'a, 'c: 'a>(image: &'b RunnableContainer, option: &'c RunOption) -> Self {
        let descriptor = image.descriptor();
        let health_check = if let WaitStrategy::CustomHealthCheck(hc) = &image.wait_strategy {
            Some(hc)
        } else {
            None
        };
        let ports = &image.port_mappings;
        let remove = option.remove;
        let name = option.name();
        let network = option
            .network
            .as_ref()
            .map_or_else(|| Cow::Owned(Network::default()), Cow::Borrowed);
        let volumes = option.volumes.as_slice();
        let env = image
            .env
            .iter()
            .map(|(key, value)| (key.as_str(), value.as_str()))
            .chain(
                option
                    .env
                    .iter()
                    .map(|(key, value)| (key.as_str(), value.as_str())),
            )
            .collect();
        let command = if let Some(cmd) = &option.command {
            cmd.as_slice()
        } else {
            image.command.as_slice()
        };
        let entrypoint = option.entrypoint.as_deref();

        Self {
            descriptor,
            health_check,
            ports,
            remove,
            name,
            network,
            volumes,
            env,
            command,
            entrypoint,
        }
    }
}

#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)]
mod tests {
    use assert2::{check, let_assert};
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case("0.0.0.0:32780", 32780)]
    #[case(
        "0.0.0.0:32780
[::]:32780
",
        32780
    )]
    fn should_parse_port(#[case] str: &str, #[case] expected: u16) {
        let result = parse_port(str);
        let_assert!(Some(port) = result);
        check!(port == expected);
    }
}
