use serde::{Deserialize, Serialize};

use super::{ContainerId, ContainerStatus};

/// Container process
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ContainerProcess {
    #[serde(alias = "ID")]
    pub(crate) id: ContainerId,
    pub(crate) names: Names,
    pub(crate) state: ContainerStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Names {
    Name(String),
    List(Vec<String>),
}

impl Names {
    pub fn contains(&self, name: &str) -> bool {
        match self {
            Self::Name(n) => n == name,
            Self::List(ns) => ns.iter().any(|n| n == name),
        }
    }
}

#[cfg(test)]
mod tests {
    use assert2::let_assert;

    use super::*;
    #[test]
    fn should_serde_docker_process() {
        let json_stream = include_str!("../../tests/assets/docker-ps.jsonl");
        let stream =
            serde_json::Deserializer::from_str(json_stream).into_iter::<ContainerProcess>();
        let result = stream.collect::<Result<Vec<_>, _>>();
        let_assert!(Ok(data) = result);
        insta::assert_debug_snapshot!(data);
    }

    #[test]
    fn should_serde_podman_process() {
        let json = include_str!("../../tests/assets/podman_ps.json");
        let result = serde_json::from_str::<Vec<ContainerProcess>>(json);
        let_assert!(Ok(data) = result);
        insta::assert_debug_snapshot!(data);
    }
}
