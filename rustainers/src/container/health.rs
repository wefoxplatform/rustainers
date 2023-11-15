use serde::{Deserialize, Serialize};
use tracing::warn;

/// The container health
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, strum_macros::Display)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ContainerHealth {
    /// Unknown
    #[default]
    Unknown,

    /// Starting (not yet healthy)
    Starting,

    /// Healthy
    Healthy,

    /// Fail to be healthy
    Unhealthy,
}

impl<'de> Deserialize<'de> for ContainerHealth {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let state = String::deserialize(deserializer)?;
        let result = match state.to_ascii_lowercase().as_str() {
            "starting" => Self::Starting,
            "healthy" => Self::Healthy,
            "unhealthy" => Self::Unhealthy,
            "unknown" => Self::Unknown,
            _ => {
                warn!(?state, "Oops, found an unknown container health");
                Self::Unknown
            }
        };

        Ok(result)
    }
}

#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)]
mod tests {

    use assert2::check;

    use super::ContainerHealth;

    #[test]
    fn should_serde_container_health() {
        let json = "\"healthy\"\n";
        let result = serde_json::from_str::<ContainerHealth>(json).unwrap();
        check!(result == ContainerHealth::Healthy);
    }
}
