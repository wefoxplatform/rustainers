use std::fmt::Display;
use std::time::Duration;

use crate::io::StdIoKind;
use crate::{ContainerStatus, HealthCheck, Port};

/// Default port scan timeout (100ms)
pub const SCAN_PORT_DEFAULT_TIMEOUT: Duration = Duration::from_millis(100);

/// Wait strategies
#[derive(Debug, Clone, Default)]
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

    /// Wait until log match a pattern
    LogMatch {
        ///
        io: StdIoKind,
        /// The matcher
        matcher: LogMatcher,
    },

    /// Do not wait
    None,
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

    /// Wait for a log line in stdout contains a string
    #[must_use]
    pub fn stdout_contains(str: impl Into<String>) -> Self {
        Self::LogMatch {
            io: StdIoKind::Out,
            matcher: LogMatcher::Contains(str.into()),
        }
    }

    /// Wait for a log line in stderr contains a string
    #[must_use]
    pub fn stderr_contains(str: impl Into<String>) -> Self {
        Self::LogMatch {
            io: StdIoKind::Err,
            matcher: LogMatcher::Contains(str.into()),
        }
    }
}

#[cfg(feature = "regex")]
impl WaitStrategy {
    /// Wait for a log line in stdout match a pattern
    #[must_use]
    pub fn stdout_match(re: regex::Regex) -> Self {
        Self::LogMatch {
            io: StdIoKind::Out,
            matcher: LogMatcher::Regex(Box::new(re)),
        }
    }

    /// Wait for a log line in stderr match a pattern
    #[must_use]
    pub fn stderr_match(re: regex::Regex) -> Self {
        Self::LogMatch {
            io: StdIoKind::Err,
            matcher: LogMatcher::Regex(Box::new(re)),
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

impl Display for WaitStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HealthCheck => write!(f, "Container health check"),
            Self::CustomHealthCheck(hc) => write!(f, "Custom health check {hc:?}"),
            Self::State(state) => write!(f, "State {state}"),
            Self::HttpSuccess {
                https,
                path,
                container_port,
            } => write!(
                f,
                "HTTP success {}on path path {path} with container port {container_port}",
                if *https { "(HTTPS)" } else { "" }
            ),
            Self::ScanPort {
                container_port,
                timeout,
            } => write!(
                f,
                "Container port {container_port} open (timeout {timeout:?})"
            ),
            Self::LogMatch { io, .. } => write!(f, "Log match pattern on {io}"),
            Self::None => write!(f, "None"),
        }
    }
}

/// The log line matcher
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum LogMatcher {
    /// The line is expected to contains the string
    Contains(String),

    #[cfg(feature = "regex")]
    /// The line is expected to match the regular expression
    Regex(Box<regex::Regex>),
}

impl LogMatcher {
    pub(crate) fn matches(&self, str: &str) -> bool {
        match self {
            Self::Contains(pattern) => str.contains(pattern),
            #[cfg(feature = "regex")]
            Self::Regex(re) => re.is_match(str),
        }
    }
}
