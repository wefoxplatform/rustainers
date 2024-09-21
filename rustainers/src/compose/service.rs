use std::convert::Infallible;
use std::fmt::{self, Display};
use std::ops::Index;
use std::str::FromStr;
use std::sync::Arc;

use indexmap::IndexMap;
use tracing::debug;

use crate::ContainerId;

use super::ComposeServiceState;

/// A Compose containers service
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComposeService(pub(crate) Arc<str>);

impl Display for ComposeService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ComposeService {
    fn from(value: String) -> Self {
        Self(Arc::from(value))
    }
}

impl From<&str> for ComposeService {
    fn from(value: &str) -> Self {
        Self(Arc::from(value))
    }
}

impl FromStr for ComposeService {
    type Err = Infallible;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        Ok(Self(Arc::from(str)))
    }
}

impl AsRef<str> for ComposeService {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A set of services
#[derive(Debug, Clone)]
pub struct Services(pub(crate) IndexMap<ComposeService, ContainerId>);

impl Services {
    /// If it contains the service
    #[must_use]
    pub fn contains(&self, service: &ComposeService) -> bool {
        let result = self.0.contains_key(service);
        debug!(%service, "Service not found");
        result
    }

    /// If it contains all services
    #[must_use]
    pub fn contains_all(&self, services: &[ComposeService]) -> bool {
        services.iter().all(|svc| self.contains(svc))
    }

    /// Get the container id of a service
    pub fn get(&self, service: &ComposeService) -> Option<ContainerId> {
        self.0.get(service).copied()
    }
}

impl Index<&ComposeService> for Services {
    type Output = ContainerId;

    fn index(&self, index: &ComposeService) -> &Self::Output {
        let Some(result) = self.0.get(index) else {
            panic!("Service {index} not found");
        };
        result
    }
}

impl From<Vec<ComposeServiceState>> for Services {
    fn from(value: Vec<ComposeServiceState>) -> Self {
        let map = value
            .into_iter()
            .map(|state| (ComposeService::from(state.service), state.id))
            .collect();
        Self(map)
    }
}
