use std::process::Command;

use rustainers::runner::{RunOption, Runner};
use rustainers::{
    ContainerStatus, ExposedPort, HealthCheck, ImageName, RunnableContainer,
    RunnableContainerBuilder, ToRunnableContainer, WaitStrategy,
};

// A web server only accessible when sharing the same network
#[derive(Debug)]
pub struct InternalWebServer;

impl ToRunnableContainer for InternalWebServer {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(ImageName::new("nginx"))
            .with_wait_strategy(
                HealthCheck::builder()
                    .with_command("curl -sf http://localhost") //DevSkim: ignore DS137138
                    .build(),
            )
            .build()
    }
}

// Curl
#[derive(Debug)]
struct Curl {
    url: String,
}

pub async fn curl(
    runner: &Runner,
    url: impl Into<String>,
    options: RunOption,
) -> anyhow::Result<()> {
    let url = url.into();
    let image = Curl { url };
    let _container = runner.start_with_options(image, options).await?;

    Ok(())
}

impl ToRunnableContainer for Curl {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(ImageName::new("curlimages/curl"))
            .with_wait_strategy(WaitStrategy::State(ContainerStatus::Exited))
            .with_command(["-fsv", "--connect-timeout", "1", &self.url])
            .build()
    }
}

// A web server accessible from host
#[derive(Debug)]
pub struct WebServer(ExposedPort);

impl Default for WebServer {
    fn default() -> Self {
        Self(ExposedPort::new(80))
    }
}

impl ToRunnableContainer for WebServer {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(ImageName::new("nginx"))
            .with_port_mappings([self.0.clone()])
            .with_wait_strategy(WaitStrategy::http("/"))
            .build()
    }
}

impl WebServer {
    // Container path that contains the static HTML pages
    pub const STATIC_HTML: &'static str = "/usr/share/nginx/html";

    /// Get the text content
    pub async fn get(&self, path: &str) -> anyhow::Result<String> {
        let port = self.0.host_port().await?;
        let url = format!("http://localhost:{port}/{}", path.trim_start_matches('/')); //DevSkim: ignore DS137138
        let out = Command::new("curl").arg(&url).output()?;
        let result = String::from_utf8_lossy(&out.stdout);
        Ok(result.to_string())
    }
}
