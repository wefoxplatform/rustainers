use std::path::PathBuf;
use std::time::Duration;

use indexmap::IndexMap;
use typed_builder::TypedBuilder;

/// Run options
#[derive(Debug, Clone, TypedBuilder)]
#[builder(doc, field_defaults(default, setter(prefix = "with_")))]
pub struct ComposeRunOption {
    /// Wait interval for service health check
    #[builder(default = Duration::from_secs(1))]
    pub(crate) wait_interval: Duration,

    /// Wait interval for service to exist
    #[builder(default = Duration::from_millis(96))]
    pub(crate) wait_services_interval: Duration,

    /// The environnement variables
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
