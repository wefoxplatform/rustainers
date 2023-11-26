use rustainers::runner::{RunOption, Runner};
use rustainers::{
    ContainerStatus, HealthCheck, ImageName, RunnableContainer, RunnableContainerBuilder,
    ToRunnableContainer, WaitStrategy,
};

// A web server
#[derive(Debug)]
pub struct WebServer;

impl ToRunnableContainer for WebServer {
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
