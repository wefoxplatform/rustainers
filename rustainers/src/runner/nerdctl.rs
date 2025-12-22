use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tracing::debug;

use super::{InnerRunner, RunnerError};
use crate::cmd::Cmd;
use crate::runner::ContainerError;
use crate::version::Version;
use crate::NetworkInfo;

const MINIMAL_VERSION: Version = Version::new(1, 5);

/// A Nerdctl runner
///
/// This runner use the nerdctl CLI
///
/// It requires nerdctl client v1.5+
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Nerdctl {
    /// The nerdctl version
    pub version: Version,
}

impl InnerRunner for Nerdctl {
    fn command(&self) -> Cmd<'static> {
        Cmd::new("nerdctl")
    }

    #[tracing::instrument(level = "debug", skip(self), fields(runner = %self))]
    async fn list_custom_networks(&self) -> Result<Vec<NetworkInfo>, ContainerError> {
        let mut cmd = self.command();

        if self.version.major > 1 && self.version.minor > 1 {
            cmd.push_args([
                "network",
                "ls",
                // Remove empty IDs.
                r#"--format='{{if ne .ID ""}}{{json .}}{{end}}'"#,
            ]);
        } else {
            cmd.push_args(["network", "ls", "--no-trunc", "--format={{json .}}"]);
        }

        let mut result = cmd.json_stream::<NetworkInfo>().await?;
        result.retain(|x| ["bridge", "host", "none"].contains(&x.name.as_str()));
        Ok(result)
    }
}

impl Display for Nerdctl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Nerdctl {}", self.version)
    }
}

pub(super) fn create() -> Result<Nerdctl, RunnerError> {
    // Check binary version
    let mut cmd = Cmd::new("nerdctl");
    cmd.push_args(["version", "--format", "json"]);
    let Ok(Some(version)) = cmd.json_blocking::<Option<NerdctlVersion>>() else {
        return Err(RunnerError::CommandNotAvailable(String::from("nerdctl")));
    };
    debug!("Found docker version: {version:#?}");

    // Check client version
    let current = version.client.version;
    if current < MINIMAL_VERSION {
        return Err(RunnerError::UnsupportedVersion {
            command: String::from("nerdctl"),
            current,
            minimal: MINIMAL_VERSION,
        });
    }

    Ok(Nerdctl { version: current })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NerdctlVersion {
    client: NerdctlClientVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NerdctlClientVersion {
    version: Version,
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_serde() {
        let json = include_str!("../../tests/assets/nerdctl_version.json");
        let version = serde_json::from_str::<NerdctlVersion>(json).expect("nerdctl version");
        insta::assert_debug_snapshot!(version);
    }

    #[cfg(feature = "ensure-nerdctl")]
    #[test]
    fn should_works() {
        _ = tracing_subscriber::fmt::try_init();
        assert2::let_assert!(Ok(_) = create());
    }
}
