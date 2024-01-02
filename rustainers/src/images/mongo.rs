use std::time::Duration;

use crate::{
    ExposedPort, HealthCheck, ImageName, Port, PortError, RunnableContainer,
    RunnableContainerBuilder, ToRunnableContainer,
};

const MONGO_IMAGE: &ImageName = &ImageName::new("mongo");

const PORT: Port = Port(27017);

/// A `Mongo` image
///
/// # Example
///
/// ```rust, no_run
/// # async fn run() -> anyhow::Result<()> {
/// use rustainers::images::Mongo;
///
/// let default_image = Mongo::default();
///
/// let custom_image = Mongo::default()
///        .with_tag("6");
///
/// # let runner = rustainers::runner::Runner::auto()?;
/// // ...
/// let container = runner.start(default_image).await?;
/// let endpoint = container.endpoint().await?;
/// // ...
/// # Ok(())
/// # }
#[derive(Debug)]
pub struct Mongo {
    image: ImageName,
    port: ExposedPort,
}

impl Mongo {
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

impl Mongo {
    /// Get endpoint URL
    ///
    /// # Errors
    ///
    /// Could fail if the port is not bind
    pub async fn endpoint(&self) -> Result<String, PortError> {
        let port = self.port.host_port().await?;
        let url = format!("mongodb://localhost:{port}");

        Ok(url)
    }
}

impl Default for Mongo {
    fn default() -> Self {
        Self {
            image: MONGO_IMAGE.clone(),
            port: ExposedPort::new(PORT),
        }
    }
}

impl ToRunnableContainer for Mongo {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(self.image.clone())
            .with_wait_strategy(
                HealthCheck::builder()
                    .with_command(
                        r#"echo 'db.runCommand("ping").ok' | mongosh localhost:27017/test --quiet"#,
                    )
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
        let image = Mongo {
            port: ExposedPort::fixed(PORT, Port::new(9123)),
            ..Default::default()
        };
        let result = image.endpoint().await;
        let_assert!(Ok(endpoint) = result);
        check!(endpoint == "mongodb://localhost:9123");
    }
}
