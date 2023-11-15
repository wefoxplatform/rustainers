use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::ContainerHealth;

/// The container State
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, strum_macros::Display,
)]
#[serde(rename_all = "lowercase")]
pub enum ContainerStatus {
    /// Unknown
    #[default]
    Unknown,

    /// Created
    Created,

    /// Running
    Running,

    /// Restarting
    Restarting,

    /// Stopped
    Stopped,

    /// Exited
    Exited,

    /// Paused
    Paused,

    /// Dead
    Dead,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ContainerState {
    #[serde(default)]
    pub(crate) status: ContainerStatus,
    #[serde(default)]
    pub(crate) health: ContainerFullStateHealth,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct ContainerFullStateHealth {
    pub(crate) status: ContainerHealth,
    #[serde(default)]
    failing_streak: usize,
    #[serde(default)]
    pub(crate) log: Option<Vec<HealthCheckLog>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct HealthCheckLog {
    start: String,
    end: String,
    exit_code: i32,
    output: String,
}

impl Display for HealthCheckLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let HealthCheckLog {
            start,
            end,
            exit_code,
            output,
        } = self;
        writeln!(f, "{start} - {end}\n{output}\nExit code: {exit_code}")
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_serde_inspect_status() {
        let json = include_str!("../../tests/assets/inspect-state.json");
        let result = serde_json::from_str::<ContainerState>(json).unwrap();
        insta::assert_debug_snapshot!(result);
    }

    #[test]
    fn should_serde_inspect_status_exited() {
        let json = include_str!("../../tests/assets/inspect-state-exited.json");
        let result = serde_json::from_str::<ContainerState>(json).unwrap();
        insta::assert_debug_snapshot!(result);
    }
}
