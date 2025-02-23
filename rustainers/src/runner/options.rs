use std::time::Duration;

use indexmap::IndexMap;
use typed_builder::TypedBuilder;

use crate::{Network, Volume};

/// Run options
///
/// Available options:
///
/// * `wait_interval`: wait until re-check a container state (default 500ms)
/// * `remove`: if we remove the container after the stop (`--rm` flag, default false)
/// * `stop_timeout`: wait after the stop before killing the container
/// * `name`: provide the container name (default unnamed, use the runner name)
/// * `network`: define the network
/// * `volumes`: set some volumes
/// * `env`: set some environment variables
#[derive(Debug, Clone, TypedBuilder)]
#[builder(field_defaults(default, setter(prefix = "with_")))]
pub struct RunOption {
    /// Wait interval for container health check
    #[builder(default = Duration::from_millis(500))]
    pub(super) wait_interval: Duration,

    /// Automatically remove the container when it exits
    pub(super) remove: bool,

    /// The duration to wait after sending the first signal to stop before killing the container
    pub(super) stop_timeout: Option<Duration>,

    /// Assign a name to the container
    #[builder(setter(into, strip_option))]
    pub(super) name: Option<String>,

    /// The network
    #[builder(default, setter(into))]
    pub(crate) network: Option<Network>,

    /// Volumes
    #[builder(default, setter(transform = |args: impl IntoIterator<Item = impl Into<Volume>>| args.into_iter().map(Into::into).collect()))]
    pub(crate) volumes: Vec<Volume>,

    /// The environment variables
    #[builder(setter(transform = |args: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>| args.into_iter().map(|(key, value)| (key.into(), value.into())).collect()))]
    pub(crate) env: IndexMap<String, String>,

    /// The command (override the runable command)
    #[builder(default, setter(transform = |args: impl IntoIterator<Item = impl Into<String>>| Some(args.into_iter().map(Into::into).collect())))]
    pub(crate) command: Option<Vec<String>>,

    /// The entrypoint (override the image entrypoint)
    #[builder(default, setter(into, strip_option))]
    pub(crate) entrypoint: Option<String>,
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

/// Stop options
///
/// Available options:
///
/// * `timeout`: wait after the stop before killing the container
#[derive(Debug, Clone, Default, TypedBuilder)]
#[builder(field_defaults(default, setter(prefix = "with_")))]
pub struct StopOption {
    /// The duration to wait after sending the first signal to stop before killing the container
    pub(super) timeout: Option<Duration>,
}
