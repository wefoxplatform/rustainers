use std::fmt::{Debug, Display};
use std::mem;
use std::net::SocketAddr;
use std::time::Duration;

use async_trait::async_trait;
use serde::de::DeserializeOwned;
use tracing::{info, trace, warn};

use crate::cmd::Cmd;
use crate::{
    ContainerHealth, ContainerId, ContainerProcess, ContainerState, ContainerStatus, Port,
    RunnableContainer, WaitStrategy,
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

    #[tracing::instrument(level = "debug", skip(self, image), fields(runner = %self, image = %image))]
    async fn create_and_start(
        &self,
        image: &RunnableContainer,
        remove: bool,
        name: Option<&str>,
    ) -> Result<ContainerId, ContainerError> {
        let mut cmd = self.command();
        cmd.push_args(["run", "--detach"]);
        let descriptor = image.descriptor();

        // --rm
        if remove {
            cmd.push_arg("--rm");
        }

        // --name
        if let Some(name) = name {
            cmd.push_args(["--name", name]);
        }

        // --env
        for (key, value) in &image.env {
            cmd.push_args([String::from("--env"), format!("{key}={value}")]);
        }

        // --publish
        for port_mapping in &image.port_mappings {
            let mapping = port_mapping.lock().await;
            let publish = mapping.to_publish();
            mem::drop(mapping);
            cmd.push_args(["--publish", &publish]);
        }

        // health check args
        if let WaitStrategy::CustomHealthCheck(hc) = &image.wait_strategy {
            cmd.push_args(hc.to_vec());
        }

        // descriptor (name:tag or other alternatives)
        cmd.push_arg(descriptor);

        // command
        cmd.push_args(&image.command);

        // Run
        info!(%image, "🚀 Launching container");
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

    #[tracing::instrument(level = "debug", skip(self, id), fields(runner = %self, id = %id))]
    async fn wait_ready(
        &self,
        id: ContainerId,
        wait_condition: &WaitStrategy,
        interval: Duration,
    ) -> Result<(), ContainerError> {
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
            }

            tokio::time::sleep(interval).await;
        }

        Ok(())
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
        let RunOption {
            wait_interval,
            remove,
            name,
        } = options;

        // Container name
        let name = name.as_deref();
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
                self.create_and_start(image, remove, container_name).await?
            }
            // Need to create and start the container
            Some((ContainerStatus::Unknown, _)) | None => {
                self.create_and_start(image, remove, container_name).await?
            }
        };

        // Wait
        self.wait_ready(id, &image.wait_strategy, wait_interval)
            .await?;

        // Port Mapping
        for port_mapping in &image.port_mappings {
            let mut mapping = port_mapping.lock().await;
            if mapping.host_port.is_none() {
                let host_port = self.port(id, mapping.container_port).await?;
                mapping.bind_port(host_port);
            }
        }

        Ok(id)
    }

    #[tracing::instrument(skip(self, id), fields(runner = %self, id = %id))]
    async fn exec(&self, id: ContainerId, exec_command: Vec<String>) -> Result<(), ContainerError> {
        let mut cmd = self.command();
        cmd.push_arg("exec");
        cmd.push_arg(id);
        cmd.push_args(exec_command);

        let stdout = cmd.result().await?;
        info!(%id, "🐚 Executed\n{stdout}",);

        Ok(())
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

fn parse_port(s: &str) -> Option<Port> {
    s.lines()
        .filter_map(|it| it.parse::<SocketAddr>().ok())
        .map(|it| Port(it.port()))
        .next()
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
    fn should_parse_port(#[case] s: &str, #[case] expected: u16) {
        let result = parse_port(s);
        let_assert!(Some(port) = result);
        check!(port == expected);
    }
}
