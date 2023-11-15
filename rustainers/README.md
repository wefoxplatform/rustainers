# wai-testcontainers

An simple, opinionated way to run containers for tests.

This crate supports [docker], [podman], or [nerdctl].

You can also run [docker compose], [podman-compose] or [nerdctl compose].

For now, the implementation is based on the CLI command. We may add more runner based on Rust api later.

The start command is `async`, and the crate is using [tokio].

## Run a simple container

You need a [`crate::runner::Runner`] to launch an image.
You can use the [`crate::runner::Runner::auto`] function to detect an available runner,
or use the [`crate::runner::Runner::docker`], [`crate::runner::Runner::podman`],
[`crate::runner::Runner::nerdctl`] functions to choose a specific runner.

Then you need to create a runnable image, see module [`crate::images`] to use an existing image,
or create your own image.

And you just need to `start` your image. The running container can provide some methods
to help you to use this container, e.g. a connection URL to access your container.

When the container is dropped, the container is stopped (unless you call detach on your container).

```rust, no_run
use rustainers::runner::{Runner, RunOption};
use rustainers::images::Postgres;

# async fn pg() -> anyhow::Result<()> {
let runner = Runner::auto()?;
let image = Postgres::default().with_tag("15.2");
let container = runner.start(image).await?;

let url = container.url()?;
do_something_with_postgres(url).await?;
# Ok(())
# }
# async fn do_something_with_postgres(url: String) -> anyhow::Result<()> { Ok(())}
```

## Create a custom image

See [`crate::images`] module documentation

[docker]: https://docs.docker.com/engine/reference/commandline/cli/
[docker compose]: https://docs.docker.com/compose/reference/
[podman]: https://docs.podman.io/en/latest/Commands.html
[podman-compose]: https://github.com/containers/podman-compose
[nerdctl]: https://github.com/containerd/nerdctl
[nerdctl compose]: https://github.com/containerd/nerdctl/blob/main/docs/compose.md
[tokio]: https://tokio.rs/
