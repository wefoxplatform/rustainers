use std::fmt::{self, Display};

use crate::compose::{
    ComposeError, RunnableComposeContainers, RunnableComposeContainersBuilder, TemporaryDirectory,
    TemporaryFile, ToRunnableComposeContainers,
};
use crate::{ExposedPort, Port, PortError};

const REDPANDA_SERVICE: &str = "redpanda-0";
const REDPANDA_PROXY_PORT: Port = Port(18082);
const REDPANDA_PORT: Port = Port(19092);
const REDPANDA_ADMIN_PORT: Port = Port(9644);

const SCHEMA_REGISTRY_PORT: Port = Port(18081);

const REDPANDA_CONSOLE_SERVICE: &str = "console";
const REDPANDA_CONSOLE_PORT: Port = Port(8080);

/// A docker compose with a single node Redpanda
#[derive(Debug)]
pub struct Redpanda {
    temp_dir: TemporaryDirectory,
    schema_registry_port: ExposedPort,
    redpanda_proxy_port: ExposedPort,
    redpanda_port: ExposedPort,
    redpanda_admin_port: ExposedPort,
    redpanda_console_port: ExposedPort,
}

impl Display for Redpanda {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Redpanda + schema registry")
    }
}

impl Redpanda {
    /// Create a [`Redpanda`]
    ///
    /// # Errors
    ///
    /// Fail if we cannot create the temporary directory
    pub async fn build_single() -> Result<Self, ComposeError> {
        let schema_registry_port = ExposedPort::new(SCHEMA_REGISTRY_PORT);
        let redpanda_proxy_port = ExposedPort::new(REDPANDA_PROXY_PORT);
        let redpanda_port = ExposedPort::new(REDPANDA_PORT);
        let redpanda_admin_port = ExposedPort::new(REDPANDA_ADMIN_PORT);
        let redpanda_console_port = ExposedPort::new(REDPANDA_CONSOLE_PORT);
        let temp_dir = TemporaryDirectory::with_files(
            "redpanda-single",
            [
                // TODO provide a macro rules for simple cases
                TemporaryFile::builder()
                    .with_path("docker-compose.yaml")
                    .with_content(include_bytes!("./docker-compose.single.yaml"))
                    .build(),
            ],
        )
        .await?;

        Ok(Self {
            temp_dir,
            schema_registry_port,
            redpanda_proxy_port,
            redpanda_port,
            redpanda_admin_port,
            redpanda_console_port,
        })
    }

    /// The Kafka broker address
    ///
    /// # Errors
    ///
    /// Fail if we cannot retrieve the Kafka host port
    pub async fn broker_address(&self) -> Result<String, PortError> {
        let port = self.redpanda_port.host_port().await?;
        let addr = format!("127.0.0.1:{port}");

        Ok(addr)
    }

    /// The schema registry endpoint
    ///
    /// # Errors
    ///
    /// Fail if we cannot retrieve the schema registry host port
    pub async fn schema_registry_endpoint(&self) -> Result<String, PortError> {
        let port = self.schema_registry_port.host_port().await?;
        let addr = format!("http://127.0.0.1:{port}");

        Ok(addr)
    }
}

impl ToRunnableComposeContainers for Redpanda {
    type AsPath = TemporaryDirectory;

    fn to_runnable(
        &self,
        builder: RunnableComposeContainersBuilder<Self::AsPath>,
    ) -> RunnableComposeContainers<Self::AsPath> {
        builder
            .with_compose_path(self.temp_dir.clone())
            .with_port_mappings([
                (REDPANDA_SERVICE, self.schema_registry_port.clone()),
                (REDPANDA_SERVICE, self.redpanda_proxy_port.clone()),
                (REDPANDA_SERVICE, self.redpanda_port.clone()),
                (REDPANDA_SERVICE, self.redpanda_admin_port.clone()),
                (REDPANDA_CONSOLE_SERVICE, self.redpanda_console_port.clone()),
            ])
            // TODO
            // .with_wait_strategies([
            // (REDPANDA_SERVICE, WaitStrategy::HealthCheck),
            // (REDPANDA_CONSOLE_SERVICE, WaitStrategy::HealthCheck),
            // ])
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_build_kafka_schema_registry() {
        _ = tracing_subscriber::fmt::try_init();

        let image = Redpanda::build_single().await.expect("red-panda");
        let dir = image.temp_dir.as_ref().to_path_buf();

        assert!(dir.join("docker-compose.yaml").exists());
    }
}
