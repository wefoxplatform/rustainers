use std::time::Duration;

use crate::{
    Container, ExposedPort, HealthCheck, ImageName, Port, PortError, RunnableContainer,
    RunnableContainerBuilder, ToRunnableContainer,
};

const REDIS_IMAGE: &ImageName = &ImageName::new("docker.io/redis");

const PORT: Port = Port(6379);

/// A `Redis` image
///
/// # Example
///
/// ```rust, no_run
/// # async fn run() -> anyhow::Result<()> {
/// use rustainers::images::Redis;
///
/// let default_image = Redis::default();
///
/// let custom_image = Redis::default()
///        .with_tag("7.2");
///
/// # let runner = rustainers::runner::Runner::auto()?;
/// // ...
/// let container = runner.start(default_image).await?;
/// let endpoint = container.endpoint().await?;
/// // ...
/// # Ok(())
/// # }
#[derive(Debug)]
pub struct Redis {
    image: ImageName,
    port: ExposedPort,
}

impl Redis {
    /// Set the image tag
    #[must_use]
    pub fn with_tag(self, tag: impl Into<String>) -> Self {
        let Self { mut image, .. } = self;
        image.set_tag(tag);
        Self { image, ..self }
    }

    /// Set the image digest
    #[must_use]
    pub fn with_digest(self, digest: impl Into<String>) -> Self {
        let Self { mut image, .. } = self;
        image.set_digest(digest);
        Self { image, ..self }
    }

    /// Set the port mapping
    #[must_use]
    pub fn with_port(mut self, port: ExposedPort) -> Self {
        self.port = port;
        self
    }
}

impl Redis {}

impl Default for Redis {
    fn default() -> Self {
        Self {
            image: REDIS_IMAGE.clone(),
            port: ExposedPort::new(PORT),
        }
    }
}

impl Container<Redis> {
    /// Get endpoint URL
    ///
    /// # Errors
    ///
    /// Could fail if the port is not bind
    pub async fn endpoint(&self) -> Result<String, PortError> {
        let port = self.port.host_port().await?;
        let host_ip = self.runner.container_host_ip().await?;
        let url = format!("redis://{host_ip}:{port}");

        Ok(url)
    }
}
impl ToRunnableContainer for Redis {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(self.image.clone())
            .with_wait_strategy(
                HealthCheck::builder()
                    .with_command("redis-cli --raw incr ping")
                    .with_start_period(Duration::from_millis(96))
                    .with_interval(Duration::from_millis(96))
                    .build(),
            )
            .with_port_mappings([self.port.clone()])
            .build()
    }
}
