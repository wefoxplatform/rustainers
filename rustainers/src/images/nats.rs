use crate::{
    Container, ExposedPort, ImageName, Port, PortError, RunnableContainer,
    RunnableContainerBuilder, ToRunnableContainer, WaitStrategy,
};

const NATS_IMAGE: &ImageName = &ImageName::new("docker.io/nats");

const CLIENT_PORT: Port = Port(4222);
const CLUSTER_PORT: Port = Port(6222);
const MONITORING_PORT: Port = Port(8222);

/// A `Nats` image
///
/// # Example
///
/// ```rust, no_run
/// # async fn run() -> anyhow::Result<()> {
/// use rustainers::images::Nats;
///
/// let default_image = Nats::default();
///
/// let custom_image = Nats::default()
///        .with_tag("2.7.4");
///
/// # let runner = rustainers::runner::Runner::auto()?;
/// // ...
/// let container = runner.start(default_image).await?;
/// let endpoint = container.client_endpoint().await?;
/// // ...
/// # Ok(())
/// # }
#[derive(Debug)]
pub struct Nats {
    image: ImageName,
    client_port: ExposedPort,
    cluster_port: ExposedPort,
    monitoring_port: ExposedPort,
}

impl Nats {
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

    /// Set the client port
    #[must_use]
    pub fn with_client_port(mut self, port: ExposedPort) -> Self {
        self.client_port = port;
        self
    }

    /// Set the cluster port
    #[must_use]
    pub fn with_cluster_port(mut self, port: ExposedPort) -> Self {
        self.cluster_port = port;
        self
    }

    /// Set the monitoring port
    #[must_use]
    pub fn with_monitoring_port(mut self, port: ExposedPort) -> Self {
        self.monitoring_port = port;
        self
    }
}

impl Default for Nats {
    fn default() -> Self {
        Self {
            image: NATS_IMAGE.clone(),
            client_port: ExposedPort::new(CLIENT_PORT),
            cluster_port: ExposedPort::new(CLUSTER_PORT),
            monitoring_port: ExposedPort::new(MONITORING_PORT),
        }
    }
}

impl Container<Nats> {
    /// Get endpoint URL for the client port
    ///
    /// # Errors
    ///
    /// Could fail if the port is not bind
    pub async fn client_endpoint(&self) -> Result<String, PortError> {
        let port = self.client_port.host_port().await?;
        let host_ip = self.runner.container_host_ip().await?;
        let url = format!("nats://{host_ip}:{port}");

        Ok(url)
    }

    /// Get endpoint URL for the monitoring port
    ///
    /// # Errors
    ///
    /// Could fail if the port is not bind
    pub async fn monitoring_endpoint(&self) -> Result<String, PortError> {
        let port = self.monitoring_port.host_port().await?;
        let host_ip = self.runner.container_host_ip().await?;
        let url = format!("http://{host_ip}:{port}");

        Ok(url)
    }

    /// Get endpoint URL for the cluster port
    ///
    /// # Errors
    ///
    /// Could fail if the port is not bind
    pub async fn cluster_endpoint(&self) -> Result<String, PortError> {
        let port = self.cluster_port.host_port().await?;
        let host_ip = self.runner.container_host_ip().await?;
        let url = format!("nats-route://{host_ip}:{port}");

        Ok(url)
    }
}

impl ToRunnableContainer for Nats {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(self.image.clone())
            .with_wait_strategy(WaitStrategy::stderr_contains(
                "Listening for client connections",
            ))
            .with_port_mappings([
                self.client_port.clone(),
                self.cluster_port.clone(),
                self.monitoring_port.clone(),
            ])
            .build()
    }
}
