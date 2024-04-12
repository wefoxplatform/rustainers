use serde::{Deserialize, Serialize};

/// The network driver
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Driver {
    /// Bridge
    Bridge,
    /// Host
    Host,
    /// Overlay
    Overlay,
    /// Ipvlan
    Ipvlan,
    /// Macvlan
    Macvlan,
    /// Null
    Null,
}

/// A runner network
///
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct RunnerNetwork {
    #[serde(alias = "name")]
    /// The network name
    pub name: String,
    #[serde(alias = "scope")]
    /// The network scope
    pub scope: Option<String>,
    #[serde(alias = "driver")]
    /// The network driver
    pub driver: Driver,
}

#[cfg(test)]
mod tests {

    use super::*;
    use assert2::let_assert;

    #[test]
    fn should_serde_network() {
        let json_stream = include_str!("../../tests/assets/docker_networks.jsonl");
        let stream = serde_json::Deserializer::from_str(json_stream).into_iter::<RunnerNetwork>();
        let networks = stream.collect::<Result<Vec<_>, _>>();
        let_assert!(Ok(data) = networks);
        insta::assert_debug_snapshot!(data);
    }

    #[test]
    fn should_serde_podman_network() {
        let json_stream = include_str!("../../tests/assets/podman_networks.jsonl");
        let stream = serde_json::Deserializer::from_str(json_stream).into_iter::<RunnerNetwork>();
        let networks = stream.collect::<Result<Vec<_>, _>>();
        let_assert!(Ok(data) = networks);
        insta::assert_debug_snapshot!(data);
    }
}
