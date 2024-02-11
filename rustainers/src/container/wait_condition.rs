use std::time::Duration;

use crate::{ContainerStatus, HealthCheck, Port};

/// Default port scan timeout (100ms)
pub const SCAN_PORT_DEFAULT_TIMEOUT: Duration = Duration::from_millis(100);

/// Wait strategies
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum WaitStrategy {
    /// With the image health check
    #[default]
    HealthCheck,

    /// With custom health check
    CustomHealthCheck(HealthCheck),

    /// Wait for the container state
    State(ContainerStatus),

    /// Wait until the HTTP call provide a successful status (e.g. 200 OK)
    HttpSuccess {
        /// If we use HTTPS instead of HTTP
        https: bool,
        /// The path to check
        path: String,
        /// The container port
        container_port: Port,
    },

    /// Wait until a socket is open
    ScanPort {
        /// The container port
        container_port: Port,
        /// The timeout for a try
        timeout: Duration,
    },

    /// Do not wait
    None,
    // TODO StdLog, ErrLog until match a regex
}

impl WaitStrategy {
    /// No wait
    #[must_use]
    pub fn none() -> Self {
        Self::None
    }

    /// Wait with image healt check
    #[must_use]
    pub fn health_check() -> Self {
        Self::HealthCheck
    }

    /// Wait with image healt check
    #[must_use]
    pub fn custom_health_check(health_check: HealthCheck) -> Self {
        Self::CustomHealthCheck(health_check)
    }

    /// Wait for a state
    #[must_use]
    pub fn state(state: ContainerStatus) -> Self {
        Self::State(state)
    }

    /// Wait for an successful HTTP call on the 80 port
    pub fn http(path: impl Into<String>) -> Self {
        let path = path.into();
        let container_port = Port(80);
        Self::HttpSuccess {
            https: false,
            path,
            container_port,
        }
    }

    /// Wait for an successful HTTPS call on the 443 port
    pub fn https(path: impl Into<String>) -> Self {
        let path = path.into();
        let container_port = Port(443);
        Self::HttpSuccess {
            https: true,
            path,
            container_port,
        }
    }

    /// Wait for a port to be open using a default timeout
    pub fn scan_port(container_port: impl Into<Port>) -> Self {
        let container_port = container_port.into();
        let timeout = SCAN_PORT_DEFAULT_TIMEOUT;
        Self::ScanPort {
            container_port,
            timeout,
        }
    }
}

impl From<HealthCheck> for WaitStrategy {
    fn from(value: HealthCheck) -> Self {
        Self::custom_health_check(value)
    }
}

impl From<ContainerStatus> for WaitStrategy {
    fn from(value: ContainerStatus) -> Self {
        Self::state(value)
    }
}
