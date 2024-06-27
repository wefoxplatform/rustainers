use ipnetwork::IpNetwork;
use std::borrow::Cow;
use std::fmt::Display;
use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

use crate::ContainerId;

/// Network settings
///
/// See [docker reference](https://docs.docker.com/engine/reference/run/#network-settings)
///
/// # Examples
///
/// ```
/// # use rustainers::{ContainerId, Network};
/// // Default network is bridge
/// assert_eq!(Network::default(), Network::Bridge);
///
/// // A network based on a container
/// let container_id = "123abc".parse::<ContainerId>().unwrap();
/// assert_eq!(Network::from(container_id), Network::Container(container_id));
///
/// // A custom network
/// assert_eq!(Network::from("my-network"), Network::Custom(String::from("my-network")));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Network {
    /// Create a network stack on the default Docker bridge
    #[default]
    Bridge,
    /// No networking
    None,
    /// Reuse another container's network stack
    Container(ContainerId), // TODO could be ContainerName
    /// Use the Docker host network stack
    Host,
    /// Connect to a user-definined network
    Custom(String),
}

impl Network {
    pub(crate) fn cmd_arg(&self) -> Cow<'static, str> {
        match self {
            Self::Bridge => Cow::Borrowed("--network=bridge"),
            Self::None => Cow::Borrowed("--network=none"),
            Self::Container(id) => Cow::Owned(format!("--network=container:{id}")),
            Self::Host => Cow::Borrowed("--network=host"),
            Self::Custom(name) => Cow::Owned(format!("--network={name}")),
        }
    }

    pub(crate) fn name(&self) -> Option<&str> {
        match self {
            Self::Bridge => Some("bridge"),
            Self::None => Some("none"),
            Self::Container(_) => None,
            Self::Host => Some("host"),
            Self::Custom(custom) => Some(custom),
        }
    }
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bridge => write!(f, "bridge"),
            Self::None => write!(f, "none"),
            Self::Container(id) => write!(f, "container:{id}"),
            Self::Host => write!(f, "host"),
            Self::Custom(custom) => write!(f, "{custom}"),
        }
    }
}

impl From<&str> for Network {
    fn from(value: &str) -> Self {
        Self::Custom(String::from(value))
    }
}

impl From<String> for Network {
    fn from(value: String) -> Self {
        Self::Custom(value)
    }
}

impl From<ContainerId> for Network {
    fn from(value: ContainerId) -> Self {
        Self::Container(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ip(pub(crate) Ipv4Addr);

mod serde_ip {
    use std::net::Ipv4Addr;

    use serde::de::Visitor;
    use serde::{Deserialize, Serialize};

    use super::Ip;

    impl Serialize for Ip {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_str(&self.0.to_string())
        }
    }

    struct IpVisitor;
    impl<'de> Visitor<'de> for IpVisitor {
        type Value = Ip;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("an IPv4 as string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            value.parse::<Ipv4Addr>().map(Ip).map_err(E::custom)
        }
    }

    impl<'de> Deserialize<'de> for Ip {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            deserializer.deserialize_str(IpVisitor)
        }
    }
}

/// A Network as described by the runner inspect command
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub(crate) struct NetworkDetails {
    #[serde(alias = "IPAddress")]
    /// Network Ip address
    pub(crate) ip_address: Option<Ip>,

    /// Network gateway
    #[serde(alias = "Gateway")]
    pub(crate) gateway: Option<Ip>,

    /// Network gateway
    #[serde(alias = "NetworkID")]
    pub(crate) id: Option<ContainerId>,
}

/// A Network as described by the runner inspect command
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub(crate) struct HostContainer {
    #[serde(alias = "Name")]
    /// Network Ip address
    pub(crate) name: Option<String>,
}

/// A Network as described by the runner network command
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NetworkInfo {
    /// Name of the network
    #[serde(alias = "Name")]
    pub(crate) name: String,

    /// Id of the network
    #[serde(alias = "ID")]
    pub(crate) id: ContainerId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct IpamNetworkConfig {
    #[serde(alias = "Subnet")]
    pub(crate) subnet: Option<IpNetwork>,

    #[serde(alias = "Gateway")]
    pub(crate) gateway: Option<Ipv4Addr>,
}

#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)]
mod tests {
    use assert2::{check, let_assert};
    use rstest::rstest;
    use std::collections::HashMap;

    use super::*;

    #[rstest]
    #[case::bridge(Network::Bridge, "--network=bridge")]
    #[case::none(Network::None, "--network=none")]
    #[case::container(Network::Container("123456".parse().expect("container id")), "--network=container:123456")]
    #[case::host(Network::Host, "--network=host")]
    #[case::custom("user-defined-net".into(), "--network=user-defined-net")]
    fn should_provide_arg(#[case] network: Network, #[case] expected: &str) {
        let arg = network.cmd_arg();
        check!(arg.as_ref() == expected);
    }

    #[test]
    fn should_deserialize_network_details() {
        let json = include_str!("../../tests/assets/docker-inspect-network.json");
        let result = serde_json::from_str::<NetworkDetails>(json);
        let_assert!(Ok(network_detail) = result);
        let ip = network_detail.ip_address.expect("IP v4").0;
        check!(ip == Ipv4Addr::from([172_u8, 29, 0, 2]));
    }

    #[test]
    fn should_deserialize_network_info() {
        let json = include_str!("../../tests/assets/docker-network.json");
        let result = serde_json::from_str::<NetworkInfo>(json);
        let_assert!(Ok(network_info) = result);
        let expected = "b79a7ee6fe69".parse::<ContainerId>();
        let_assert!(Ok(expected_id) = expected);
        check!(network_info.id == expected_id);
    }

    #[test]
    fn should_deserialize_host_containers() {
        let json = include_str!("../../tests/assets/docker-inspect-containers.json");
        let result = serde_json::from_str::<HashMap<ContainerId, HostContainer>>(json);
        let_assert!(Ok(containers) = result);
        let id = "f7bbcdb277f7cc880b84219c959a5d28169ebb8c41dd32c08a9195a3c79e8d5e"
            .parse::<ContainerId>();
        let_assert!(Ok(container_id) = id);
        let_assert!(Some(host) = containers.get(&container_id));
        let_assert!(Some(container_name) = &host.name);
        check!(container_name == &"dockerindocker".to_string());
    }
}
