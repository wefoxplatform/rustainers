use std::path::Path;

use typed_builder::TypedBuilder;

use super::ComposeService;
use crate::{SharedExposedPort, WaitStrategy};

/// Contains configuration require to run compose containers
#[derive(Debug, Clone, TypedBuilder)]
#[builder(doc, field_defaults(setter(prefix = "with_")))]
#[non_exhaustive]
pub struct RunnableComposeContainers<P> {
    pub(crate) compose_path: P,

    /// The wait condition
    #[builder(default, setter(transform = |args: impl IntoIterator<Item = (impl Into<ComposeService>, impl Into<WaitStrategy>)>| args.into_iter().map(|(k,v)| (k.into(), v.into())).collect()))]
    pub(crate) wait_strategies: Vec<(ComposeService, WaitStrategy)>,

    /// The services port mapping
    #[builder(default, setter(transform = |args: impl IntoIterator<Item = (impl Into<ComposeService>, SharedExposedPort)>| args.into_iter().map(|(k,v)| (k.into(), v)).collect()))]
    pub(crate) port_mappings: Vec<(ComposeService, SharedExposedPort)>,
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
