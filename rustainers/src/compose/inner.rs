use std::ffi::OsStr;
use std::path::Path;
use std::time::Duration;

use async_trait::async_trait;
use tracing::info;

use crate::cmd::Cmd;
use crate::runner::InnerRunner;
use crate::{ContainerId, ExposedPort, WaitStrategy};

use super::{ComposeError, ComposeRunOption, ComposeService, ComposeServiceState, Services};

#[async_trait]
pub(crate) trait InnerComposeRunner: InnerRunner {
    fn compose_command(&self) -> Result<Cmd<'static>, ComposeError>;

    #[tracing::instrument(skip(self), fields(runner = %self))]
    async fn start_compose(
        &self,
        dir: &Path,
        wait_strategies: &[(ComposeService, WaitStrategy)],
        port_mappings: &mut [(ComposeService, ExposedPort)],
        options: ComposeRunOption,
    ) -> Result<String, ComposeError> {
        let Some(name) = dir.file_name().and_then(OsStr::to_str).map(str::to_string) else {
            return Err(ComposeError::BadComposeFile(dir.to_path_buf()))?;
        };
        self.compose_up(&name, dir, &options).await?;

        // Find required services
        let required_services = wait_strategies
            .iter()
            .map(|(svc, _)| svc.clone())
            .collect::<Vec<_>>();
        let services = self
            .find_required_services(
                &name,
                &required_services,
                options.wait_services_interval,
                dir,
            )
            .await?;

        // Wait
        let interval = options.wait_interval;
        for (service, wait) in wait_strategies {
            debug_assert!(services.contains(service));
            #[allow(clippy::indexing_slicing)]
            let id = services[service];
            self.wait_service_ready(service, id, wait, interval).await?;
        }

        // Port mapping
        for (service, mapping) in port_mappings {
            debug_assert!(services.contains(service));
            #[allow(clippy::indexing_slicing)]
            let id = services[service];
            let port = self.port(id, mapping.container_port).await?;
            mapping.bind_port(port).await;
        }

        Ok(name)
    }

    #[tracing::instrument(level = "debug", skip(self), fields(runner = %self))]
    async fn wait_service_ready(
        &self,
        service: &ComposeService,
        id: ContainerId,
        wait_condition: &WaitStrategy,
        interval: Duration,
    ) -> Result<(), ComposeError> {
        self.wait_ready(id, wait_condition, interval).await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self), fields(runner = %self))]
    async fn compose_up(
        &self,
        name: &str,
        dir: &Path,
        options: &ComposeRunOption,
    ) -> Result<(), ComposeError> {
        info!(%name, ?dir, "🚀 Launching compose container");
        let mut cmd = self.compose_command()?;
        cmd.with_dir(dir);
        cmd.push_args(["up", "--detach"]);
        if let Some(file) = options.compose_file.as_ref().and_then(|it| it.to_str()) {
            cmd.push_args(["--file", file]);
        }
        cmd.set_env(options.env.clone());

        let cmd_err = cmd.clone();
        let status = cmd.status().await?;
        if status.success() {
            Ok(())
        } else {
            Err(ComposeError::ComposeContainerCannotBeStarted(
                cmd_err.to_string(),
            ))
        }
    }

    async fn compose_look_up_services(
        &self,
        _name: &str,
        path: &Path,
    ) -> Result<Services, ComposeError> {
        let mut cmd = self.compose_command()?;
        cmd.with_dir(path);
        cmd.push_args(["ps", "--all", "--no-trunc", "--format", "json"]);
        let states = cmd.json_stream::<ComposeServiceState>().await?;
        let result = Services::from(states);
        Ok(result)
    }

    #[tracing::instrument(skip(self), fields(runner = %self))]
    async fn find_required_services(
        &self,
        name: &str,
        required_services: &[ComposeService],
        interval: Duration,
        path: &Path,
    ) -> Result<Services, ComposeError> {
        loop {
            let result = self.compose_look_up_services(name, path).await?;
            if result.contains_all(required_services) {
                return Ok(result);
            }
            tokio::time::sleep(interval).await;
        }
    }
}
