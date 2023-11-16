use crate::Port;

/// Port error
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum PortError {
    /// Invalid port mapping
    #[error("Invalid port mapping, expect a `<host port>:<container port>`, got {0}")]
    InvalidPortMapping(String),

    /// The port is not yet bind
    #[error("Container port {0} not bind")]
    PortNotBindYet(Port),
}
