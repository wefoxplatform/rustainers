use std::time::Duration;

use typed_builder::TypedBuilder;

/// Run options
///
/// Available options:
///
/// * `wait_interval`: wait until re-check a container state (default 500ms)
/// * `remove`: if we remove the container after the stop (`--rm` flag, default false)
/// * `name`: provide the container name (default unnamed, use the runner name)
#[derive(Debug, Clone, TypedBuilder)]
#[builder(field_defaults(default, setter(prefix = "with_")))]
pub struct RunOption {
    /// Wait interval for container health check
    #[builder(default = Duration::from_millis(500))]
    pub(super) wait_interval: Duration,

    /// Automatically remove the container when it exits
    pub(super) remove: bool,

    /// Assign a name to the container
    #[builder(setter(into, strip_option))]
    pub(super) name: Option<String>,
}

impl RunOption {
    /// If we need to remove the container when it's stopped
    #[must_use]
    pub fn remove(&self) -> bool {
        self.remove
    }

    /// The container name
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl Default for RunOption {
    fn default() -> Self {
        RunOption::builder().build()
    }
}
