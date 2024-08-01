use crate::{
    Container, ExposedPort, ImageName, Port, PortError, RunnableContainer,
    RunnableContainerBuilder, ToRunnableContainer, WaitStrategy,
};

const MONGO_IMAGE: &ImageName = &ImageName::new("docker.io/mongo");

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

    /// Set the port mapping
    #[must_use]
    pub fn with_port(mut self, port: ExposedPort) -> Self {
        self.port = port;
        self
    }
}

impl Container<Mongo> {
    /// Get endpoint URL
    ///
    /// # Errors
    ///
    /// Could fail if the port is not bind
    pub async fn endpoint(&self) -> Result<String, PortError> {
        let port = self.port.host_port().await?;
        let host_ip = self.runner.container_host_ip().await?;
        let url = format!("mongodb://{host_ip}:{port}");

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
            .with_wait_strategy(WaitStrategy::stdout_contains("Waiting for connections"))
            .with_port_mappings([self.port.clone()])
            .build()
    }
}
