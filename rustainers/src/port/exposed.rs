use std::str::FromStr;
use std::sync::Arc;

use tokio::sync::Mutex;

use tracing::debug;

use super::{Port, PortError};

/// Define an exposed port
///
/// An exposed port has two parts: the container port (inside), and the host port (outside).
/// To talk to the container you need to use the host port.
///
/// # Examples
///
/// You can omit the host port during construction, in that case, a default
/// available port is chosen. For example, to create an exposed port
/// targeting the container `80` port:
///
/// ```rust
/// # use rustainers::ExposedPort;
/// let port_mapping = ExposedPort::new(80);
/// ```
///
/// You can set the host port, but be careful it can fail it's already opened.
/// For example, to create the exposed host port `8080`
/// targeting the container `80` port:
///
/// ```rust
/// # use rustainers::ExposedPort;
/// let port_mapping = ExposedPort::fixed(80, 8080);
/// ```
#[derive(Debug, Clone)]
pub struct ExposedPort {
    pub(crate) container_port: Port,
    pub(crate) host_port: Arc<Mutex<Option<Port>>>,
}

impl ExposedPort {
    /// Create an exposed port
    pub fn new(container_port: impl Into<Port>) -> ExposedPort {
        Self {
            container_port: container_port.into(),
            host_port: Arc::default(),
        }
    }

    /// Create an exposed port with a fixed host port
    pub fn fixed(container_port: impl Into<Port>, host_port: impl Into<Port>) -> ExposedPort {
        Self {
            container_port: container_port.into(),
            host_port: Arc::new(Mutex::new(Some(host_port.into()))),
        }
    }

    /// Get the bound port (host)
    ///
    /// # Errors
    ///
    /// Fail if the port is unbound
    pub async fn host_port(&self) -> Result<Port, PortError> {
        let port = self.host_port.lock().await;
        port.ok_or(PortError::PortNotBindYet(self.container_port))
    }

    /// Get the container port
    #[must_use]
    pub fn container_port(&self) -> Port {
        self.container_port
    }

    pub(crate) async fn to_publish(&self) -> String {
        let port = self.host_port.lock().await;
        port.map_or(self.container_port.to_string(), |host| {
            format!("{host}:{}", self.container_port)
        })
    }

    /// Bind the host port (if it's not already bound)
    pub(crate) async fn bind_port(&mut self, host_port: Port) {
        let mut port = self.host_port.lock().await;
        if port.is_none() {
            *port = Some(host_port);
            debug!(%host_port, container_port=%self.container_port, "bound port");
        }
    }
}

impl FromStr for ExposedPort {
    type Err = PortError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let Some((host, container)) = str.split_once(':') else {
            return Err(PortError::InvalidPortMapping(str.to_string()));
        };
        let host_port = host
            .parse()
            .map_err(|_| PortError::InvalidPortMapping(str.to_string()))?;
        let container_port = container
            .parse()
            .map_err(|_| PortError::InvalidPortMapping(str.to_string()))?;

        Ok(Self {
            host_port: Arc::new(Mutex::new(Some(host_port))),
            container_port,
        })
    }
}

#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)]
mod tests {
    use assert2::{check, let_assert};

    use super::*;

    #[tokio::test]
    async fn should_parse_exposed_port() {
        let str = "1234:80";
        let result = str.parse::<ExposedPort>().expect("port");
        check!(result.container_port() == 80);
        check!(result.host_port().await.expect("host port") == 1234);
    }

    #[rstest::rstest]
    #[case::empty("")]
    #[case::only_one("1234")]
    #[case::bad_separator("1234->80")]
    #[case::empty_port("1234:")]
    #[case::invalid_first_port("a:80")]
    #[case::invalid_second_port("1234:a")]
    fn should_not_parse_invalid_exposed_port(#[case] str: &str) {
        let result = str.parse::<ExposedPort>();
        let_assert!(Err(PortError::InvalidPortMapping(s2)) = result);
        check!(str == s2);
    }

    #[tokio::test]
    async fn should_bind_port() {
        const CONTAINER: u16 = 42;
        let host = 1324;
        let mut exposed_port = ExposedPort::new(CONTAINER);

        // should fail if no host
        let result = exposed_port.host_port().await;
        let_assert!(Err(_) = result);

        // bind the good port
        exposed_port.bind_port(Port(host)).await;
        check!(exposed_port.host_port().await.expect("host port") == host);

        // should fail if no host
        let result = exposed_port.host_port().await;
        let_assert!(Ok(Port(host_port)) = result);
        check!(host_port == host);
    }
}
