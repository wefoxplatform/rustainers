use std::path::PathBuf;
use std::time::Duration;

use indexmap::IndexMap;
use typed_builder::TypedBuilder;

/// Run options
///
/// Available options:
///
/// * `wait_interval`: wait until re-check a container state (default 1s)
/// * `wait_services_interval`: wait until re-check that all services starting (default 96ms)
/// * `env`: a map of environment variables used when launch the container
/// * `compose-file`: if you need to use another compose file (`--file` option)
#[derive(Debug, Clone, TypedBuilder)]
#[builder(field_defaults(default, setter(prefix = "with_")))]
pub struct ComposeRunOption {
    /// Wait interval for service health check
    #[builder(default = Duration::from_secs(1))]
    pub(crate) wait_interval: Duration,

    /// Wait interval for service to exist
    #[builder(default = Duration::from_millis(96))]
    pub(crate) wait_services_interval: Duration,

    /// The environment variables
    #[builder(setter(transform = |args: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>| args.into_iter().map(|(k,v)| (k.into(), v.into())).collect()))]
    pub(crate) env: IndexMap<String, String>,

    /// The compose file
    #[builder(setter(strip_option))]
    pub(crate) compose_file: Option<PathBuf>,
    // TODO Wait timeout
}

impl Default for ComposeRunOption {
    fn default() -> Self {
        ComposeRunOption::builder().build()
    }
}
