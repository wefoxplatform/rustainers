use serde::{Deserialize, Serialize};

use crate::{ContainerHealth, ContainerId, ContainerStatus};

/// A compose service state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ComposeServiceState {
    #[serde(alias = "ID")]
    pub(super) id: ContainerId,
    name: String,
    pub(super) service: String,
    state: ContainerStatus,
    health: ContainerHealth,
    exit_code: Option<i32>,
}
