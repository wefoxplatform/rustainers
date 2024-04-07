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
    /// The network name
    pub name: String,
    /// The network scope
    pub scope: String,
    /// The network driver
    pub driver: Driver,
}
impl RunnerNetwork {
    #[allow(dead_code)]
    #[must_use]
    /// Check network is from the host
    ///
    pub fn is_host_bridge_network(&self) -> bool {
        self.scope == "local" && self.driver == Driver::Bridge && self.name != "bridge"
    }
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
    fn should_filter_network() {
        let json_stream = include_str!("../../tests/assets/docker_networks.jsonl");
        let stream = serde_json::Deserializer::from_str(json_stream).into_iter::<RunnerNetwork>();
        let networks = stream.collect::<Result<Vec<_>, _>>().unwrap();
        let host_network = networks
            .into_iter()
            .filter(RunnerNetwork::is_host_bridge_network)
            .collect::<Vec<RunnerNetwork>>();
        assert_eq!(host_network.len(), 1);
    }
}
