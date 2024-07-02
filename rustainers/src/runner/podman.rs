use std::fmt::Display;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{debug, info};

use crate::cmd::Cmd;
use crate::version::Version;
use crate::ContainerId;
use crate::ContainerProcess;
use crate::IpamNetworkConfig;
use crate::NetworkInfo;

use super::{ContainerError, InnerRunner, RunnerError};
const MINIMAL_VERSION: Version = Version::new(4, 0);
const COMPOSE_MINIMAL_VERSION: Version = Version::new(1, 0);

/// A Podman runner
///
/// This runner use the podman CLI
///
/// It requires podman client v4.0+
///
/// podman-compose is supported if v1.0+
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Podman {
    /// The client version
    pub version: Version,

    /// The podman-compose version
    pub compose_version: Option<Version>,
}

#[async_trait]
impl InnerRunner for Podman {
    fn command(&self) -> Cmd<'static> {
        Cmd::new("podman")
    }

    #[tracing::instrument(level = "info", skip(self), fields(runner = %self))]
    fn is_inside_container(&self) -> bool {
        Path::new("/run/.containerenv").exists()
    }

    #[tracing::instrument(level = "debug", skip(self), fields(runner = %self))]
    async fn list_custom_networks(&self) -> Result<Vec<NetworkInfo>, ContainerError> {
        let mut cmd: Cmd<'_> = self.command();
        cmd.push_args(["network", "ls", "--no-trunc", "--format={{json .}}"]);
        let mut result = cmd.json_stream::<NetworkInfo>().await?;
        result.retain(|x| "podman" == x.name);
        Ok(result)
    }

    #[tracing::instrument(level = "debug", skip(self), fields(runner = %self))]
    async fn list_network_config(
        &self,
        network_id: ContainerId,
    ) -> Result<Vec<IpamNetworkConfig>, ContainerError> {
        self.inspect(network_id, ".Subnets").await
    }

    #[tracing::instrument(level = "debug", skip(self), fields(runner = %self))]
    async fn ps(&self, name: &str) -> Result<Option<ContainerProcess>, ContainerError> {
        let mut cmd = self.command();
        cmd.push_args([
            "ps",
            "--all",
            "--no-trunc",
            "--filter",
            &format!("name={name}"),
            "--format=json",
        ]);

        let containers = cmd.json::<Vec<ContainerProcess>>().await?;
        let result = containers.into_iter().find(|it| it.names.contains(name));
        Ok(result)
    }
}

impl Display for Podman {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Podman {}", self.version)?;
        if let Some(compose_version) = self.compose_version {
            write!(f, " - podman-compose {compose_version}")?;
        }
        Ok(())
    }
}
pub(super) fn create() -> Result<Podman, RunnerError> {
    // Check binary version
    let mut cmd = Cmd::new("podman");
    cmd.push_args(["version", "--format", "json"]);
    let Ok(Some(version)) = cmd.json_blocking::<Option<PodmanVersion>>() else {
        return Err(RunnerError::CommandNotAvailable(String::from("podman")));
    };

    // Check client version
    let current = version.client.api_version;
    debug!("Found podman version: {current}");
    if current < MINIMAL_VERSION {
        return Err(RunnerError::UnsupportedVersion {
            command: String::from("podman"),
            current,
            minimal: MINIMAL_VERSION,
        });
    }

    let compose_version = compose_version();

    Ok(Podman {
        version: current,
        compose_version,
    })
}

fn compose_version() -> Option<Version> {
    // Check the help command not fail
    let mut cmd = Cmd::new("podman-compose");
    cmd.ignore_stderr();
    cmd.push_args(["version", "--format", "json"]);
    let Ok(result) = cmd.result_blocking() else {
        debug!("Fail to check podman-compose version");
        return None;
    };

    let Ok(Some(compose_version)) = extract_podman_compose_version(&result) else {
        debug!("Invalid podman-compose version, {result}");
        return None;
    };

    // Check minimal version
    let version = compose_version.version;
    debug!("Podman compose version: {version}");
    if version < COMPOSE_MINIMAL_VERSION {
        info!(
            "Podman compose version {version} is not supported, require to be >= {COMPOSE_MINIMAL_VERSION}"
        );
        return None;
    }

    Some(version)
}

fn extract_podman_compose_version(
    output: &str,
) -> Result<Option<PodmanComposeVersion>, serde_json::Error> {
    let Some(last_line) = output.trim().lines().last() else {
        debug!("Fail to retrieve podman-compose version");
        return Ok(None);
    };

    let result = serde_json::from_str::<PodmanComposeVersion>(last_line)?;
    Ok(Some(result))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PodmanVersion {
    client: PodmanVersionItem,
    server: Option<PodmanVersionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PodmanComposeVersion {
    version: Version,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct PodmanVersionItem {
    #[serde(rename = "APIVersion")]
    api_version: Version,
    version: Version,
}

#[cfg(test)]
mod tests {

    use assert2::let_assert;

    use super::*;

    #[test]
    fn should_serde() {
        let json = include_str!("../../tests/assets/podman_version.json");
        let version = serde_json::from_str::<PodmanVersion>(json).expect("podman version");
        let result = serde_json::to_string_pretty(&version).expect("json");
        insta::assert_snapshot!(result);
    }
    #[test]
    fn should_serde_compose() {
        let output = include_str!("../../tests/assets/podman-compose_version.txt");
        let result = extract_podman_compose_version(output);
        let_assert!(Ok(Some(version)) = result);
        insta::assert_debug_snapshot!(version);
    }

    #[cfg(feature = "ensure-podman")]
    #[test]
    fn should_works() {
        _ = tracing_subscriber::fmt::try_init();
        assert2::let_assert!(Ok(_) = create());
    }
}
