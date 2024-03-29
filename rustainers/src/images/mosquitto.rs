use std::time::Duration;

use crate::{
    ExposedPort, ImageName, Port, PortError, RunnableContainer,
    RunnableContainerBuilder, ToRunnableContainer, WaitStrategy,
};

const MOSQUITTO_IMAGE: &ImageName = &ImageName::new("eclipse-mosquitto");

const PORT: Port = Port(6379);

/// A `mosquitto` image
///
/// # Example
///
/// ```rust, no_run
/// # async fn run() -> anyhow::Result<()> {
/// use rustainers::images::Mosquitto;
///
/// let default_image = Mosquitto::default();
///
/// let custom_image = Mosquitto::default()
///        .with_tag("2.0.18");
///
/// # let runner = rustainers::runner::Runner::auto()?;
/// // ...
/// let container = runner.start(default_image).await?;
/// let endpoint = container.endpoint().await?;
/// // ...
/// # Ok(())
/// # }
#[derive(Debug)]
pub struct Mosquitto {
    image: ImageName,
    port: ExposedPort,
}

impl Mosquitto {
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
}

impl Mosquitto {
    /// Get endpoint URL
    ///
    /// # Errors
    ///
    /// Could fail if the port is not bind
    pub async fn endpoint(&self) -> Result<String, PortError> {
        let port = self.port.host_port().await?;
        let url = format!("mqtt://localhost:{port}");

        Ok(url)
    }
}

impl Default for Mosquitto {
    fn default() -> Self {
        Self {
            image: MOSQUITTO_IMAGE.clone(),
            port: ExposedPort::new(PORT),
        }
    }
}

impl ToRunnableContainer for Mosquitto {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(self.image.clone())
            .with_command(["mosquitto", "-c", "/mosquitto-no-auth.conf"])
            .with_wait_strategy(WaitStrategy::ScanPort {
                container_port: PORT,
                timeout: Duration::from_secs(10),
            })
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
        let image = Mosquitto {
            port: ExposedPort::fixed(PORT, Port::new(9123)),
            ..Default::default()
        };
        let result = image.endpoint().await;
        let_assert!(Ok(endpoint) = result);
        check!(endpoint == "mqtt://localhost:9123");
    }
}
