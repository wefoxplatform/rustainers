use crate::{
    ImageName, RunnableContainer, RunnableContainerBuilder, ToRunnableContainer, WaitStrategy,
};

/// An alpine image.
///
/// Typically use with [`crate::tools`]
#[derive(Debug)]
pub struct Alpine;

impl ToRunnableContainer for Alpine {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(ImageName::new("docker.io/alpine"))
            .with_wait_strategy(WaitStrategy::None)
            // keep the container alive
            .with_command(["tail", "-f", "/dev/null"])
            .build()
    }
}
