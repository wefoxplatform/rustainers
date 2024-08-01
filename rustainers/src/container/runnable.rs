use std::fmt::{self, Display};

use indexmap::IndexMap;
use typed_builder::TypedBuilder;

use crate::{ExposedPort, ImageReference, WaitStrategy};

/// Contains configuration require to create and run a container
///
/// # Example
///
/// ```rust
/// # use std::time::Duration;
/// # use rustainers::{RunnableContainer, HealthCheck, ExposedPort, ImageName};
/// let runnable = RunnableContainer::builder()
///     .with_image(ImageName::new("docker.io/redis"))
///     .with_wait_strategy(
///         HealthCheck::builder()
///             .with_command("redis-cli --raw incr ping")
///             .with_start_period(Duration::from_millis(96))
///             .with_interval(Duration::from_millis(96))
///             .build(),
///     )
///     .with_port_mappings([ExposedPort::new(6379)])
///     .build();
/// ```
///
/// See [existings images](https://github.com/wefoxplatform/rustainers/tree/main/rustainers/src/images/)
/// for real usages.
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

    /// The environment variables
    #[builder(default, setter(transform = |args: impl IntoIterator<Item = (impl Into<String>, impl Into<String>)>| args.into_iter().map(|(key, value)| (key.into(), value.into())).collect()))]
    pub(crate) env: IndexMap<String, String>,

    /// The wait strategy
    #[builder(default, setter(into))]
    pub(crate) wait_strategy: WaitStrategy,

    /// The ports mapping
    #[builder(default, setter(transform = |args: impl IntoIterator<Item = ExposedPort>| args.into_iter().collect()))]
    pub(crate) port_mappings: Vec<ExposedPort>,
}

impl RunnableContainer {
    /// Build the descriptor of an image (name + tag)
    #[must_use]
    pub fn descriptor(&self) -> String {
        self.image.to_string()
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
