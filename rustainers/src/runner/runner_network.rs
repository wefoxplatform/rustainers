use serde::{Deserialize, Serialize};
use std::str::FromStr;

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
// #[strum()]
pub struct RunnerNetwork {
    /// The network name
    name: String,
    scope: String,
    driver: Driver,
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

/// A list of runner networks
///
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerNetworks {
    /// The networks
    pub networks: Vec<RunnerNetwork>,
}

impl FromStr for RunnerNetworks {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Convert to string to get the lines
        let input = s.to_string();
        let networks = input
            .lines()
            .map(serde_json::from_str)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RunnerNetworks { networks })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn should_serde_network() {
        let json = include_str!("../../tests/assets/docker_networks.jsonl");
        let networks = json.parse::<RunnerNetworks>().unwrap();
        insta::assert_debug_snapshot!(networks);
    }

    #[test]
    fn should_filter_network() {
        let json = include_str!("../../tests/assets/docker_networks.jsonl");
        let networks = json.parse::<RunnerNetworks>().unwrap();
        let host_network = networks
            .networks
            .into_iter()
            .filter(RunnerNetwork::is_host_bridge_network)
            .collect::<Vec<RunnerNetwork>>();
        assert_eq!(host_network.len(), 1);
    }
}
