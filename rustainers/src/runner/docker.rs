use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::cmd::Cmd;
use crate::runner::RunnerError;
use crate::version::Version;

use super::InnerRunner;

const MINIMAL_VERSION: Version = Version::new(1, 20);
const COMPOSE_MINIMAL_VERSION: Version = Version::new(2, 6);

/// A Docker runner
///
/// This runner use the docker CLI
///
/// It requires docker client v1.20+
///
/// Docker compose should be v2.10+
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Docker {
    /// The docker client version
    pub version: Version,
    /// The docker compose client version
    pub compose_version: Option<Version>,
}

impl InnerRunner for Docker {
    fn command(&self) -> Cmd<'static> {
        Cmd::new("docker")
    }
}

impl Display for Docker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Docker {}", self.version)?;
        if let Some(compose_version) = self.compose_version {
            write!(f, " - compose {compose_version}")?;
        }
        Ok(())
    }
}

pub(super) fn create() -> Result<Docker, RunnerError> {
    // Check binary version
    let mut cmd = Cmd::new("docker");
    cmd.push_args(["version", "--format", "{{json .}}"]);
    let Ok(Some(version)) = cmd.json_blocking::<Option<DockerVersion>>() else {
        return Err(RunnerError::CommandNotAvailable(String::from("docker")));
    };

    // Check client version
    let current = version.client.api_version;
    debug!("Found docker version: {current}");
    if current < MINIMAL_VERSION {
        return Err(RunnerError::UnsupportedVersion {
            command: String::from("docker"),
            current,
            minimal: MINIMAL_VERSION,
        });
    }

    let compose_version = compose_version();

    Ok(Docker {
        version: current,
        compose_version,
    })
}

fn compose_version() -> Option<Version> {
    let mut cmd = Cmd::new("docker");
    cmd.push_args(["compose", "version", "--format", "json"]);
    let Ok(Some(docker_compose_version)) = cmd.json_blocking::<Option<DockerComposeVersion>>()
    else {
        debug!("Fail to check docker compose version");
        return None;
    };

    // Check minimal version
    let version = docker_compose_version.version;
    debug!("Docker compose version: {version}");
    if version < COMPOSE_MINIMAL_VERSION {
        info!(
            "Docker compose version {version} is not supported, require to be >= {COMPOSE_MINIMAL_VERSION}"
        );
        return None;
    }
    Some(version)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DockerVersion {
    client: DockerVersionItem,
    server: Option<DockerVersionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct DockerVersionItem {
    api_version: Version,
    version: Version,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DockerComposeVersion {
    version: Version,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_serde() {
        let json = include_str!("../../tests/assets/docker_version.json");
        let version = serde_json::from_str::<DockerVersion>(json).expect("docker version");
        insta::assert_debug_snapshot!(version);
    }
    #[test]
    fn should_serde_compose() {
        let json = include_str!("../../tests/assets/docker-compose_version.json");
        let version =
            serde_json::from_str::<DockerComposeVersion>(json).expect("docker compose version");
        insta::assert_debug_snapshot!(version);
    }

    #[cfg(feature = "ensure-docker")]
    #[test]
    fn should_works() {
        _ = tracing_subscriber::fmt::try_init();
        assert2::let_assert!(Ok(_) = create());
    }
}
