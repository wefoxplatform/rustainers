use std::fmt::{self, Display};

use indexmap::IndexMap;
use typed_builder::TypedBuilder;

use crate::{ImageReference, SharedExposedPort, WaitStrategy};

/// Contains configuration require to create and run a container
#[derive(Debug, TypedBuilder)]
#[builder(field_defaults(setter(prefix = "with_")))]
#[non_exhaustive]
pub struct RunnableContainer {
    /// The container image
    #[builder(setter(into))]
    pub(crate) image: ImageReference,

    /// The container name
    #[builder(default, setter(into))]
    pub(crate) container_name: Option<String>,

    /// The command
    #[builder(default, setter(transform = |args: impl IntoIterator<Item = impl Into<String>>| args.into_iter().map(Into::into).collect()))]
    pub(crate) command: Vec<String>,

    /// The environnement variables
    #[builder(default, setter(transform = |args: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>| args.into_iter().map(|(k,v)| (k.into(), v.into())).collect()))]
    pub(crate) env: IndexMap<String, String>,

    /// The wait strategy
    #[builder(default, setter(into))]
    pub(crate) wait_strategy: WaitStrategy,

    /// The ports mapping
    #[builder(default, setter(transform = |args: impl IntoIterator<Item = SharedExposedPort>| args.into_iter().collect()))]
    pub(crate) port_mappings: Vec<SharedExposedPort>,
    // TODO networks
    // TODO volumes
    // TODO entrypoint
}

impl RunnableContainer {
    /// Build the descriptor of an image (name + tag)
    #[must_use]
    pub fn descriptor(&self) -> String {
        match &self.image {
            ImageReference::Id(id) => id.to_string(),
            ImageReference::Name(name) => name.to_string(),
        }
    }
}

impl Display for RunnableContainer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.descriptor())
    }
}

/// Provide an [`RunnableContainer`] and configure ports
///
/// A container image should implement this trait.
/// See [`crate::images`] for usage.
// TODO implement this trait for a single docker file with build
// TODO derive macro?
pub trait ToRunnableContainer {
    /// Build the runnable container
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer;
}
