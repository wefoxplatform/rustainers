# Provide a compose runnable images

To run compose with the runner we need to implements the [`ToRunnableComposeContainers`](crate::compose::ToRunnableComposeContainers).

It's a little bit different of the trait for the runnable container because we need:

- something that provides a reference on a [`std::path::Path`]
- wait strategies are associated with a service name
- port mapping are associated with a service name

The provided path should contains the docker-compose file.

We can use a [`TemporaryDirectory`](crate::compose::TemporaryDirectory) that provide the path.
This effective directory is created at runtime in the temporary folder, 
and when the compose containers is dropped this folder is removed.
For debugging purpose, if you need to keep this directory, you can call 
[`TemporaryDirectory::detach`](crate::compose::TemporaryDirectory::detach).

# Custom compose containers

A compose that only have the nginx service:


```rust, no_run
use std::process::Command;

use tracing::Level;

use rustainers::compose::{
    RunnableComposeContainers, RunnableComposeContainersBuilder, TemporaryDirectory, TemporaryFile,
    ToRunnableComposeContainers,
};
use rustainers::runner::Runner;
use rustainers::{ExposedPort, PortError, WaitStrategy};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runner = Runner::auto()?;
    let image = ComposeNginx::new().await;
    let containers = runner.compose_start(image).await?;

    // Making a dummy HTTP request
    let url = containers.url().await?;
    Command::new("curl").args(["-v", &url]).status()?;

    Ok(())
}

#[derive(Debug)]
struct ComposeNginx {
    temp_dir: TemporaryDirectory,
    nginx_port: ExposedPort,
}

impl ComposeNginx {

    async fn new() -> Self {
        // Create a temporary directory what will be removed when the containers stop
        let temp_dir = TemporaryDirectory::with_files(
            "componse-nginx", // a name prefix
            [
            // One docker-compose.yaml file
            TemporaryFile::builder()
                .with_path("docker-compose.yaml")
                .with_content( // you can use the include_str! macro
                    r#"
version: "3.7"

services:
    nginx:
        image: nginx
        ports:
            - 80
        healthcheck:
            test: ["CMD", "curl", "-sf", "http://127.0.0.1"]
            interval: 1s
            retries: 5
            start_period: 1s
"#,
                )
                .build()],
        )
        .await
        .expect("Should create temp_dir");

        let nginx_port = ExposedPort::new(80);
        Self { temp_dir, nginx_port }
    }

    /// Build the URL of the running nginx
    async fn url(&self) -> Result<String, PortError> {
        let port = self.nginx_port.host_port().await?;
        let url = format!("http://127.0.0.1:{port}");

        Ok(url)
    }
}

impl ToRunnableComposeContainers for ComposeNginx {
    type AsPath = TemporaryDirectory;

    fn to_runnable(
        &self,
        builder: RunnableComposeContainersBuilder<Self::AsPath>,
    ) -> RunnableComposeContainers<Self::AsPath> {
        builder
            .with_compose_path(self.temp_dir.clone())
            // Use the service healthcheck
            .with_wait_strategies([("nginx", WaitStrategy::HealthCheck)])
            .with_port_mappings([("nginx", self.nginx_port.clone())])
            .build()
    }
}
```

<div class="warning">
DO NOT use the top-level <code>name</code> of the docker-compose file.
The default folder name is used instead of this name.
</div>
