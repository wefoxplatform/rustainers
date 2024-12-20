use std::time::Duration;

use crate::runner::RunnerError;
use crate::{
    Container, ExposedPort, HealthCheck, ImageName, Port, PortError, RunnableContainer,
    RunnableContainerBuilder, ToRunnableContainer,
};

const DATA: &str = "/data";

const MINIO_IMAGE: &ImageName = &ImageName::new("docker.io/minio/minio");

const PORT: Port = Port(9000);

const CONSOLE_PORT: Port = Port(9001);

/// A `Minio` image
///
/// # Example
///
/// ```rust, no_run
/// # async fn run() -> anyhow::Result<()> {
/// use rustainers::images::Minio;
///
/// let default_image = Minio::default();
///
/// let custom_image = Minio::default()
///        .with_tag("RELEASE.2023-10-25T06-33-25Z");
///
/// # let runner = rustainers::runner::Runner::auto()?;
/// // ...
/// let container = runner.start(default_image).await?;
/// let endpoint = container.endpoint().await?;
/// // ...
/// # Ok(())
/// # }
///```
#[derive(Debug)]
pub struct Minio {
    image: ImageName,
    port: ExposedPort,
    console_port: ExposedPort,
}

impl Minio {
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

impl Minio {
    /// The region
    #[must_use]
    pub fn region(&self) -> &'static str {
        "us-east-1"
    }

    /// The access key id
    #[must_use]
    pub fn access_key_id(&self) -> &'static str {
        "minioadmin"
    }

    /// The secret access key
    #[must_use]
    pub fn secret_access_key(&self) -> &'static str {
        "minioadmin"
    }
}

impl Container<Minio> {
    /// Create a bucket
    ///
    /// # Errors
    ///
    /// Could fail if we cannot create the bucket
    pub async fn create_s3_bucket(&self, name: &str) -> Result<(), RunnerError> {
        let bucket = format!("{DATA}/{name}");
        self.runner.exec(self, ["mc", "mb", &bucket]).await?;
        self.runner
            .exec(self, ["mc", "anonymous", "set", "public", &bucket])
            .await?;

        Ok(())
    }

    /// Get endpoint URL
    ///
    /// # Errors
    ///
    /// Could fail if the port is not bind
    pub async fn endpoint(&self) -> Result<String, PortError> {
        let port = self.port.host_port().await?;

        let host_ip = self.runner.container_host_ip().await?;
        let url = format!("http://{host_ip}:{port}");

        Ok(url)
    }

    /// Get console endpoint URL
    ///
    /// # Errors
    ///
    /// Could fail if the console port is not bind
    pub async fn console_endpoint(&self) -> Result<String, PortError> {
        let port = self.console_port.host_port().await?;
        let host_ip = self.runner.container_host_ip().await?;
        let url = format!("http://{host_ip}:{port}");

        Ok(url)
    }
}

impl Default for Minio {
    fn default() -> Self {
        Minio {
            image: MINIO_IMAGE.clone(),
            port: ExposedPort::new(PORT),
            console_port: ExposedPort::new(CONSOLE_PORT),
        }
    }
}

impl ToRunnableContainer for Minio {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(self.image.clone())
            .with_wait_strategy({
                HealthCheck::builder()
                    .with_command("mc ping --exit --json local".to_string())
                    .with_interval(Duration::from_millis(250))
                    .build()
            })
            .with_command(["server", DATA])
            .with_port_mappings([self.port.clone(), self.console_port.clone()])
            .build()
    }
}
