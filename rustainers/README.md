[![Build status](https://github.com/wefoxplatform/rustainers/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/wefoxplatform/rustainers/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/rustainers)](https://crates.io/crates/rustainers)
[![Documentation](https://docs.rs/rustainers/badge.svg)](https://docs.rs/rustainers)

# rustainers

`rustainers` is a simple, opinionated way to run containers for tests.

## TL;DR

More information about this crate can be found in the [crate documentation][docs].

Or just see [example directory][examples].

## Differences with testcontainers

This crate is an alternative to the [testcontainers-rs] crate.

The key differences are

- this crate supports [docker], [podman], or [nerdctl]
- this crate supports [docker compose], [podman-compose] or [nerdctl compose]
- the start command is `async`, and the crate is using [tokio]

For now, the implementation is based on the CLI command. 
We may add more runner based on Rust api later.

## Run a simple container

You need a [`Runner`](crate::runner::Runner) to launch an image.
You can use the [`Runner::auto`](crate::runner::Runner::auto) function to detect an available runner,
or use the [`Runner::docker`](crate::runner::Runner::docker), [`Runner::podman`](crate::runner::Runner::podman),
[`Runner::nerdctl`](crate::runner::Runner::nerdctl) functions to choose a specific runner.

Then you need to create a runnable image, see module [`images`](crate::images) to use an existing image,
or create your own image.

And you just need to `start` your image. The running container can provide some methods
to help you to use this container, e.g. a connection URL to access your container.

```rust, no_run
use rustainers::runner::{Runner, RunOption};
use rustainers::images::Postgres;

async fn pg() -> anyhow::Result<()> {
    let runner = Runner::auto()?;
    let image = Postgres::default().with_tag("15.2");
    let container = runner.start(image).await?;

    let url = container.url().await?;
    do_something_with_postgres(url).await?;
    Ok(())
}

async fn do_something_with_postgres(url: String) -> anyhow::Result<()> { 
    Ok(())
}
```

## Lifecycle of a container

When you start a _runnable image_, the runner first check the state of the container.
It may already exists, or we may need to create it.

Then, the runner wait until all _wait strategies_ to be reached,
for example it can check the health of the container.

And it's returning a _container_.
This container have a reference on the _runnable image_,
this image can provide helper functions to retrieve for example the database URL.

When the _container_ is dropped, by default, it stop the container.

You can opt-in to _detach_ the container to avoid stopping in during the drop.

## Using compose

To run containers defined with a docker-compose file, you also need a runner.

Then you can start your containers with `compose_start`.

```rust, no_run
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use tracing::{info, Level};

use rustainers::compose::images::KafkaSchemaRegistry;
use rustainers::runner::Runner;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runner = Runner::auto()?;
    let image = KafkaSchemaRegistry::build_single_kraft().await?;
    let containers = runner.compose_start(image).await?;
    info!("Now I can use {containers}");
    do_something_with_kafka(&containers).await?;

   Ok(())
}

async fn do_something_with_kafka(image: &KafkaSchemaRegistry) -> anyhow::Result<()> {
    let topic = "plop";
    let broker_address = image.broker_address().await?;
    info!("Using kafka broker: {broker_address}");

    let schema_registry_url = image.schema_registry_endpoint().await?;
    info!("Using schema registry: {schema_registry_url}");

    // ...
    Ok(())
}
```

## Create a custom image

See [`images`](crate::images) module documentation.


## Create a custom compose images

See [`compose::images`](crate::compose::images) module documentation.

[docker]: https://docs.docker.com/engine/reference/commandline/cli/
[docker compose]: https://docs.docker.com/compose/reference/
[podman]: https://docs.podman.io/en/latest/Commands.html
[podman-compose]: https://github.com/containers/podman-compose
[nerdctl]: https://github.com/containerd/nerdctl
[nerdctl compose]: https://github.com/containerd/nerdctl/blob/main/docs/compose.md
[tokio]: https://tokio.rs/
[testcontainers-rs]: https://github.com/testcontainers/testcontainers-rs
[docs]: https://docs.rs/rustainers
[examples]: https://github.com/wefoxplatform/rustainers/tree/main/rustainers/examples
