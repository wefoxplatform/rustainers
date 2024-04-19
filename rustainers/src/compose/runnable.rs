use std::path::Path;

use typed_builder::TypedBuilder;

use super::ComposeService;
use crate::{ExposedPort, WaitStrategy};

/// Contains configuration require to run compose containers
///
/// See [existings compose images](https://github.com/wefoxplatform/rustainers/tree/main/rustainers/src/compose/images/)
/// for real usages.
#[derive(Debug, Clone, TypedBuilder)]
#[builder(field_defaults(setter(prefix = "with_")))]
#[non_exhaustive]
pub struct RunnableComposeContainers<P> {
    pub(crate) compose_path: P,

    /// The wait condition
    #[builder(default, setter(transform = |args: impl IntoIterator<Item = (impl Into<ComposeService>, impl Into<WaitStrategy>)>| args.into_iter().map(|(key, value)| (key.into(), value.into())).collect()))]
    pub(crate) wait_strategies: Vec<(ComposeService, WaitStrategy)>,

    /// The services port mapping
    #[builder(default, setter(transform = |args: impl IntoIterator<Item = (impl Into<ComposeService>, ExposedPort)>| args.into_iter().map(|(key, value)| (key.into(), value)).collect()))]
    pub(crate) port_mappings: Vec<(ComposeService, ExposedPort)>,
}

/// Build a runnable compose containers
pub trait ToRunnableComposeContainers {
    /// The path type
    type AsPath: AsRef<Path>;

    /// Should provide the path of the docker-compose file
    ///
    /// # Errors
    ///
    /// If the compose container cannot be created (e.g. issue while creating a [`crate::compose::TemporaryDirectory`])
    fn to_runnable(
        &self,
        builder: RunnableComposeContainersBuilder<Self::AsPath>,
    ) -> RunnableComposeContainers<Self::AsPath>;
}
