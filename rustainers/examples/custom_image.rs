use std::process::Command;

use tracing::{info, Level};

use rustainers::runner::{RunOption, Runner};
use rustainers::{
    ExposedPort, HealthCheck, ImageName, RunnableContainer, RunnableContainerBuilder,
    ToRunnableContainer,
};

mod common;
pub use self::common::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing(Level::INFO);

    let runner = Runner::auto()?;
    let options = RunOption::builder()
        .with_remove(true)
        .with_name("plop-nginx")
        .build();

    let image = Nginx::default();
    let container = runner.start_with_options(image, options).await?;
    info!("Now I can use {container}");

    // Making a dummy HTTP request
    let port = container.port.host_port()?;
    let url = format!("http://localhost:{port}"); //DevSkim: ignore DS137138
    Command::new("curl").args(["-v", &url]).status()?;

    Ok(())
}

const NGINX_IMAGE: &ImageName = &ImageName::new("nginx");

const PORT: u16 = 80;

#[derive(Debug, Clone)]
struct Nginx {
    image: ImageName,
    port: ExposedPort,
}

impl Default for Nginx {
    fn default() -> Self {
        Self {
            image: NGINX_IMAGE.clone(),
            port: ExposedPort::new(PORT),
        }
    }
}

impl ToRunnableContainer for Nginx {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(self.image.clone())
            .with_wait_strategy(
                HealthCheck::builder()
                    .with_command("curl -sf http://localhost") //DevSkim: ignore DS137138
                    .build(),
            )
            .with_port_mappings([self.port])
            .build()
    }
}
