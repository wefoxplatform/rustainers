use std::time::Duration;

use typed_builder::TypedBuilder;

/// A custom health check
///
/// # Example
///
/// ```rust
/// # use rustainers::HealthCheck;
/// # use std::time::Duration;
/// let hc = HealthCheck::builder()
///     .with_command("redis-cli --raw incr ping")
///     .with_start_period(Duration::from_millis(96))
///     .with_interval(Duration::from_millis(96))
///     .build();
/// ```
///
/// Note that the command is executed inside the container
// TODO maybe a macro rules can help to create the Heathcheck?
#[derive(Debug, Clone, PartialEq, Eq, TypedBuilder)]
#[builder(field_defaults(setter(prefix = "with_")))]
pub struct HealthCheck {
    /// Command to run to check health
    #[builder(setter(into))]
    command: String,

    /// Time between running the check
    #[builder(default = Duration::from_secs(1))]
    interval: Duration,

    /// Consecutive failures needed to report unhealthy
    #[builder(default = 10)]
    retries: u32,

    /// Start period for the container to initialize before starting health-retries countdown
    #[builder(default = Duration::from_secs(1))]
    start_period: Duration,

    /// Maximum time to allow one check to run
    #[builder(default = Duration::from_secs(30))]
    timeout: Duration,
}

impl HealthCheck {
    pub(crate) fn to_vec(&self) -> Vec<String> {
        vec![
            format!("--health-cmd={}", self.command),
            format!("--health-interval={}ms", self.interval.as_millis()),
            format!("--health-retries={}", self.retries),
            format!("--health-start-period={}ms", self.start_period.as_millis()),
            format!("--health-timeout={}ms", self.timeout.as_millis()),
        ]
    }
}
