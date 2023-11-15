use std::time::Duration;

use crate::runner::RunnerError;
use crate::{
    Container, ExposedPort, HealthCheck, ImageName, Port, PortError, RunnableContainer,
    RunnableContainerBuilder, ToRunnableContainer,
};

const DATA: &str = "/data";

const MINIO_IMAGE: &ImageName = &ImageName::new("minio/minio");

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
/// let endpoint = container.endpoint()?;
/// // ...
/// # Ok(())
/// # }
///```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Minio {
    image: ImageName,
    port: ExposedPort,
    console_port: ExposedPort,
}

impl Minio {
    /// Set the image tag
    #[must_use]
    pub fn with_tag(self, tag: impl Into<String>) -> Self {
        let Self {
            mut image,
            port,
            console_port,
        } = self;
        image.set_tag(tag);
        Self {
            image,
            port,
            console_port,
        }
    }

    /// Set the image digest
    #[must_use]
    pub fn with_digest(self, digest: impl Into<String>) -> Self {
        let Self {
            mut image,
            port,
            console_port,
        } = self;
        image.set_digest(digest);
        Self {
            image,
            port,
            console_port,
        }
    }
}

impl Minio {
    /// The region
    #[must_use]
    pub fn region(&self) -> &str {
        "us-east-1"
    }

    /// The access key id
    #[must_use]
    pub fn access_key_id(&self) -> &str {
        "minioadmin"
    }

    /// The secret access key
    #[must_use]
    pub fn secret_access_key(&self) -> &str {
        "minioadmin"
    }

    /// Get endpoint URL
    ///
    /// # Errors
    ///
    /// Could fail if the port is not bind
    pub fn endpoint(&self) -> Result<String, PortError> {
        let port = self.port.host_port()?;
        let url = format!("http://localhost:{port}");

        Ok(url)
    }

    /// Get console endpoint URL
    ///
    /// # Errors
    ///
    /// Could fail if the console port is not bind
    pub fn console_endpoint(&self) -> Result<String, PortError> {
        let port = self.console_port.host_port()?;
        let url = format!("http://localhost:{port}");

        Ok(url)
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
            .with_port_mappings([self.port, self.console_port])
            .build()
    }
}

#[cfg(test)]
#[allow(clippy::ignored_unit_patterns)]
mod tests {
    use super::*;
    use assert2::{check, let_assert};

    #[test]
    fn should_create_endpoint() {
        let image = Minio {
            port: ExposedPort::fixed(PORT, Port::new(9123)),
            ..Default::default()
        };
        let result = image.endpoint();
        let_assert!(Ok(endpoint) = result);
        check!(endpoint == "http://localhost:9123");
    }
}
