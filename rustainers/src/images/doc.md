# Runnable images

This module contains runnable images that can be started by a [`Runner`](crate::runner::Runner).

These images implements the [`ToRunnableContainer`](crate::ToRunnableContainer) trait.

## Create a custom runnable image

A runnable image should implement the [`ToRunnableContainer`](crate::ToRunnableContainer) trait.

```rust, no_run
use std::fmt::Display;

use rustainers::runner::{RunOption, Runner};
use rustainers::{
    ExposedPort, HealthCheck, RunnableContainer, RunnableContainerBuilder, Port, ToRunnableContainer,
    ImageName,
};

// Declare the image as a constant.
// You can provide a tag or a digest if you want.
const NGINX_IMAGE: &ImageName = &ImageName::new("nginx");

const PORT: u16 = 80;

/// The NGinx image
#[derive(Debug)]
struct Nginx {
    /// The image name
    image: ImageName,
    /// The exposed port
    port: ExposedPort,
}

// Provide an easy way to create the image instance
impl Default for Nginx {
    fn default() -> Self {
        Self {
            image: NGINX_IMAGE.clone(),
            port: ExposedPort::new(PORT), // the container port
        }
    }
}

// You had to implement the `ToRunnableContainer` trait
impl ToRunnableContainer for Nginx {

    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            // provide the image
            .with_image(self.image.clone())
            // strategy to check when container is ready
            .with_wait_strategy(
                // here a `curl` is enough
                // Note that this command is executed in the container
                // therefore you need to have the `curl` command available in the container
                HealthCheck::builder()
                    .with_command("curl -sf http://localhost") //DevSkim: ignore DS137138
                    .build(),
            )
            // ports mapping
            // bound a random port available port of the host to the container `80` port
            .with_port_mappings([self.port.clone()])
            .build()
    }
}
```

⚠️ WARNING: when you use an `ExposedPort`, do not make your image clonable.
The `ExposedPort` use [__Interior mutability__](https://doc.rust-lang.org/reference/interior-mutability.html).
If you clone this exposed port, it will not create a new unassign port, but share the same one.
If you really need the `Clone`, implement the trait manually with creating new `ExposedPort` instead of clone it.
