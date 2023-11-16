use std::fmt::{self, Display};
use std::str::FromStr;
use std::sync::Arc;

use tokio::sync::Mutex;

use tracing::debug;

use super::{Port, PortError};

/// A shared exposed port (interior mutability)
pub type SharedExposedPort = Arc<Mutex<ExposedPort>>;

/// Define an exposed port
///
/// # Examples
///
/// Create an exposed port targeting the container `80` port:
///
/// ```rust
/// # use rustainers::ExposedPort;
/// let port_mapping = ExposedPort::shared(80);
/// ```
///
/// Create the exposed host port `8080` targeting the container `80` port:
///
/// ```rust
/// # use rustainers::ExposedPort;
/// let port_mapping = ExposedPort::fixed(80, 8080);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExposedPort {
    pub(crate) container_port: Port,
    pub(crate) host_port: Option<Port>,
}

impl ExposedPort {
    /// Create an exposed port
    pub fn shared(container_port: impl Into<Port>) -> SharedExposedPort {
        let result = Self {
            container_port: container_port.into(),
            host_port: None,
        };
        Arc::new(Mutex::new(result))
    }

    /// Create an exposed port with a fixed host port
    pub fn fixed(container_port: impl Into<Port>, host_port: impl Into<Port>) -> SharedExposedPort {
        let result = Self {
            container_port: container_port.into(),
            host_port: Some(host_port.into()),
        };
        Arc::new(Mutex::new(result))
    }

    /// Get the bound port (host)
    ///
    /// # Errors
    ///
    /// Fail if the port is unbound
    pub fn host_port(self) -> Result<Port, PortError> {
        self.host_port
            .ok_or(PortError::PortNotBindYet(self.container_port))
    }

    /// Get the container port
    #[must_use]
    pub fn container_port(self) -> Port {
        self.container_port
    }

    pub(crate) fn to_publish(self) -> String {
        if let Some(host) = self.host_port {
            format!("{host}:{}", self.container_port)
        } else {
            self.container_port.to_string()
        }
    }

    /// Bind the host port (if it's not already bound)
    pub(crate) fn bind_port(&mut self, host_port: Port) {
        if self.host_port.is_none() {
            self.host_port = Some(host_port);
            debug!(port=%self, "bound port");
        }
    }
}

impl FromStr for ExposedPort {
    type Err = PortError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((host, container)) = s.split_once(':') else {
            return Err(PortError::InvalidPortMapping(s.to_string()));
        };
        let host_port = host
            .parse()
            .map_err(|_| PortError::InvalidPortMapping(s.to_string()))?;
        let container_port = container
            .parse()
            .map_err(|_| PortError::InvalidPortMapping(s.to_string()))?;

        Ok(Self {
            host_port: Some(host_port),
            container_port,
        })
    }
}

impl Display for ExposedPort {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(port) = self.host_port {
            write!(f, "{port} -> {}", self.container_port)
        } else {
            write!(f, "unbound ({})", self.container_port)
        }
    }
}

#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)]
mod tests {
    use assert2::{check, let_assert};

    use super::*;

    #[test]
    fn should_parse_exposed_port() {
        let s = "1234:80";
        let result = s.parse::<ExposedPort>().unwrap();
        check!(
            result
                == ExposedPort {
                    host_port: Some(Port::new(1234)),
                    container_port: Port::new(80),
                }
        );
    }

    #[rstest::rstest]
    #[case::empty("")]
    #[case::only_one("1234")]
    #[case::bad_separator("1234->80")]
    #[case::empty_port("1234:")]
    #[case::invalid_first_port("a:80")]
    #[case::invalid_second_port("1234:a")]
    fn should_not_parse_invalid_exposed_port(#[case] s: &str) {
        let result = s.parse::<ExposedPort>();
        let_assert!(Err(PortError::InvalidPortMapping(s2)) = result);
        check!(s == s2);
    }

    #[tokio::test]
    async fn should_bind_port() {
        const CONTAINER: u16 = 42;
        let host = 1324;
        let exposed_port = ExposedPort::shared(CONTAINER);
        let mut shared = exposed_port.lock().await;

        // should fail if no host
        let result = shared.host_port();
        let_assert!(Err(_) = result);

        // bind the good port
        shared.bind_port(Port(host));
        check!(shared.host_port == Some(Port(host)));

        // should fail if no host
        let result = shared.host_port();
        let_assert!(Ok(Port(h)) = result);
        check!(h == host);
    }
}
