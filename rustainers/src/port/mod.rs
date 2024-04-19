use std::fmt::Display;
use std::str::FromStr;

mod error;
pub use self::error::*;

mod exposed;
pub use self::exposed::*;

/// A Port
///
/// # Example
///
/// You can create a port from an `u16`:
///
/// ```rust
/// # use rustainers::Port;
/// let port = Port::from(8080);
///```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Port(pub(super) u16);

impl Port {
    /// Create a port
    #[must_use]
    pub const fn new(port: u16) -> Self {
        Self(port)
    }
}

impl From<u16> for Port {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<Port> for u16 {
    fn from(value: Port) -> Self {
        value.0
    }
}

impl From<Port> for String {
    fn from(value: Port) -> Self {
        value.0.to_string()
    }
}

impl Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Port {
    type Err = PortError;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let port = str
            .parse()
            .map_err(|_| PortError::InvalidPortMapping(str.to_string()))?;
        Ok(Self(port))
    }
}

impl PartialEq<u16> for Port {
    fn eq(&self, other: &u16) -> bool {
        self.0 == *other
    }
}
