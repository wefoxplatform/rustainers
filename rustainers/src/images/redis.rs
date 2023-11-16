use std::time::Duration;

use crate::{
    ExposedPort, HealthCheck, ImageName, Port, PortError, RunnableContainer,
    RunnableContainerBuilder, ToRunnableContainer,
};

const REDIS_IMAGE: &ImageName = &ImageName::new("redis");

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
        let Self { mut image, port } = self;
        image.set_tag(tag);
        Self { image, port }
    }

    /// Set the image digest
    #[must_use]
    pub fn with_digest(self, digest: impl Into<String>) -> Self {
        let Self { mut image, port } = self;
        image.set_digest(digest);
        Self { image, port }
    }
}

impl Redis {
    /// Get endpoint URL
    ///
    /// # Errors
    ///
    /// Could fail if the port is not bind
    pub async fn endpoint(&self) -> Result<String, PortError> {
        let port = self.port.host_port().await?;
        let url = format!("redis://localhost:{port}");

        Ok(url)
    }
}

impl Default for Redis {
    fn default() -> Self {
        Self {
            image: REDIS_IMAGE.clone(),
            port: ExposedPort::new(PORT),
        }
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

#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)]
mod tests {

    use super::*;
    use assert2::{check, let_assert};

    #[tokio::test]
    async fn should_create_endpoint() {
        let image = Redis {
            port: ExposedPort::fixed(PORT, Port::new(9123)),
            ..Default::default()
        };
        let result = image.endpoint().await;
        let_assert!(Ok(endpoint) = result);
        check!(endpoint == "redis://localhost:9123");
    }
}
