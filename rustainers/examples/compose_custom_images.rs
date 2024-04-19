//! Example to show how to build a custom image
#![allow(clippy::expect_used)]

use std::process::Command;

use tracing::Level;

use rustainers::compose::{
    RunnableComposeContainers, RunnableComposeContainersBuilder, TemporaryDirectory, TemporaryFile,
    ToRunnableComposeContainers,
};
use rustainers::runner::Runner;
use rustainers::{ExposedPort, PortError, WaitStrategy};

mod common;
pub use self::common::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing(Level::DEBUG);

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
        let temp_dir = TemporaryDirectory::with_files(
            "componse-nginx",
            [TemporaryFile::builder()
                .with_path("docker-compose.yaml")
                .with_content(
                    r#"
version: "3.7"

services:
    nginx:
        image: nginx
        ports:
            - 80
        healthcheck:
            test: ["CMD", "curl", "-sf", "http://localhost"]
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
        Self {
            temp_dir,
            nginx_port,
        }
    }

    async fn url(&self) -> Result<String, PortError> {
        let port = self.nginx_port.host_port().await?;
        let url = format!("http://localhost:{port}");

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
            .with_wait_strategies([("nginx", WaitStrategy::HealthCheck)])
            .with_port_mappings([("nginx", self.nginx_port.clone())])
            .build()
    }
}
