use crate::{ContainerStatus, HealthCheck};

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
    // TODO Socket until available (from container)
    // nc -z localhost 9092 || exit 1

    // TODO readiness URL (from container)
    // curl --fail http://localhost:8081/ || exit 1

    // TODO StdLog, ErrLog until match a regex
}

impl From<HealthCheck> for WaitStrategy {
    fn from(value: HealthCheck) -> Self {
        Self::CustomHealthCheck(value)
    }
}

impl From<ContainerStatus> for WaitStrategy {
    fn from(value: ContainerStatus) -> Self {
        Self::State(value)
    }
}
