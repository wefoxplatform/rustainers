use std::path::Path;

use tracing::{info, warn};

use crate::runner::{Runner, RunnerError};

use super::{
    ComposeContainers, ComposeError, ComposeRunOption, InnerComposeRunner,
    RunnableComposeContainers, ToRunnableComposeContainers,
};

impl Runner {
    /// Run a compose container with the default options
    ///
    /// # Errors
    ///
    /// Fail if the compose containers cannot be started
    pub async fn compose_start<I>(&self, images: I) -> Result<ComposeContainers<I>, RunnerError>
    where
        I: ToRunnableComposeContainers,
    {
        let options = ComposeRunOption::default();
        self.compose_start_with_options(images, options).await
    }

    /// Run compose containers with options
    ///
    /// # Errors
    ///
    /// Fail if the compose containers cannot be started
    pub async fn compose_start_with_options<I>(
        &self,
        images: I,
        options: ComposeRunOption,
    ) -> Result<ComposeContainers<I>, RunnerError>
    where
        I: ToRunnableComposeContainers,
    {
        let mut containers = images.to_runnable(RunnableComposeContainers::builder());
        let file = containers.compose_path.as_ref();
        let wait = &containers.wait_strategies;
        let mappings = &mut containers.port_mappings;

        let name = match self {
            Runner::Docker(runner) => runner.start_compose(file, wait, mappings, options).await,
            Runner::Podman(runner) => runner.start_compose(file, wait, mappings, options).await,
            Runner::Nerdctl(runner) => runner.start_compose(file, wait, mappings, options).await,
        }
        .map_err(|source| RunnerError::ComposeError {
            runner: self.clone(),
            path: file.to_path_buf(),
            source: Box::new(source),
        })?;

        Ok(ComposeContainers {
            runner: self.clone(),
            name,
            images,
            file: containers.compose_path,
            detached: false,
        })
    }

    pub(crate) fn compose_stop(&self, name: &str, file: &Path) -> Result<(), ComposeError> {
        if !file.exists() {
            return Err(ComposeError::ComposeFileMissing(file.to_path_buf()));
        }
        let mut cmd = match self {
            Runner::Docker(runner) => runner.compose_command()?,
            Runner::Podman(runner) => runner.compose_command()?,
            Runner::Nerdctl(runner) => runner.compose_command()?,
        };
        cmd.with_dir(file);
        cmd.push_args(["down"]);
        let status = cmd.status_blocking()?;
        if status.success() {
            info!(%name, "🛑 Compose containers stopped");
        } else {
            warn!(%name, ?status, "⚠️ Fail to stop compose containers");
        }
        Ok(())
    }
}

mod docker {
    use std::path::Path;

    use crate::cmd::Cmd;
    use crate::compose::{ComposeError, ComposeServiceState, InnerComposeRunner, Services};
    use crate::runner::{Docker, InnerRunner};
    use crate::version::Version;

    // https://docs.docker.com/compose/release-notes/#2210
    const PS_JSON_LINES_MINIMAL_VERSION: Version = Version::new(2, 21);

    // See <https://docs.docker.com/compose/release-notes/#2230>
    const NO_TRUNC_MINIMAL_VERSION: Version = Version::new(2, 23);

    impl InnerComposeRunner for Docker {
        fn compose_command(&self) -> Result<Cmd<'static>, ComposeError> {
            if self.compose_version.is_none() {
                return Err(ComposeError::UnsupportedComposeCommand(self.to_string()));
            };
            let mut cmd = self.command();
            cmd.push_arg("compose");
            cmd.ignore_stderr();
            Ok(cmd)
        }

        async fn compose_look_up_services(
            &self,
            _name: &str,
            path: &Path,
        ) -> Result<Services, ComposeError> {
            let mut cmd = self.compose_command()?;
            cmd.with_dir(path);
            let compose_version = self
                .compose_version
                .ok_or(ComposeError::MissingComposeVersion)?;

            let services = if compose_version >= NO_TRUNC_MINIMAL_VERSION {
                cmd.push_args(["ps", "--all", "--no-trunc", "--format", "json"]);
                cmd.json_stream::<ComposeServiceState>().await?
            } else if compose_version >= PS_JSON_LINES_MINIMAL_VERSION {
                cmd.push_args(["ps", "--all", "--format", "json"]);
                cmd.json_stream::<ComposeServiceState>().await?
            } else {
                cmd.push_args(["ps", "--all", "--format", "json"]);
                cmd.json::<Vec<ComposeServiceState>>().await?
            };
            let result = Services::from(services);

            Ok(result)
        }
    }
}

mod nerdctl {

    use crate::cmd::Cmd;
    use crate::compose::{ComposeError, InnerComposeRunner};
    use crate::runner::{InnerRunner, Nerdctl};

    impl InnerComposeRunner for Nerdctl {
        fn compose_command(&self) -> Result<Cmd<'static>, ComposeError> {
            let mut cmd = self.command();
            cmd.push_arg("compose");
            Ok(cmd)
        }
    }
}

mod podman {
    use std::path::Path;

    use serde::{Deserialize, Serialize};

    use crate::cmd::Cmd;
    use crate::compose::{ComposeError, ComposeService, InnerComposeRunner, Services};
    use crate::runner::{InnerRunner, Podman};
    use crate::{ContainerHealth, ContainerId, ContainerStatus};

    impl InnerComposeRunner for Podman {
        fn compose_command(&self) -> Result<Cmd<'static>, ComposeError> {
            if self.compose_version.is_none() {
                return Err(ComposeError::UnsupportedComposeCommand(self.to_string()));
            };
            let mut cmd = Cmd::new("podman-compose");
            cmd.ignore_stderr();
            Ok(cmd)
        }

        async fn compose_look_up_services(
            &self,
            name: &str,
            _path: &Path,
        ) -> Result<Services, ComposeError> {
            // To use the JSON output, we need to use the standard ps command of podman
            let mut cmd = self.command();
            let label = format!(
                "label=io.podman.compose.project={}",
                name.to_ascii_lowercase()
            );
            cmd.push_args(["ps", "--all", "--filter", &label, "--format", "json"]);
            let containers = cmd.json::<Vec<PodmanComposeServiceState>>().await?;
            let result = containers
                .into_iter()
                .map(|it| (ComposeService::from(it.labels.service), it.id))
                .collect();
            Ok(Services(result))
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct PodmanComposeServiceState {
        pub(crate) id: ContainerId,
        names: Vec<String>,
        labels: PodmanComposeLabels,
        state: ContainerStatus,
        health: Option<ContainerHealth>,
        exit_code: Option<i32>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PodmanComposeLabels {
        #[serde(rename = "com.docker.compose.container-number")]
        container_number: String,
        #[serde(rename = "com.docker.compose.project")]
        project: String,
        #[serde(rename = "com.docker.compose.project.config_files")]
        config_files: String,
        #[serde(rename = "com.docker.compose.project.working_dir")]
        working_dir: String,
        #[serde(rename = "com.docker.compose.service")]
        service: String,
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn should_serde_podman_service() {
            let json = include_str!("../../tests/assets/podman_lookup.json");
            let services =
                serde_json::from_str::<Vec<PodmanComposeServiceState>>(json).expect("json");
            insta::assert_json_snapshot!(services);
        }
    }
}
