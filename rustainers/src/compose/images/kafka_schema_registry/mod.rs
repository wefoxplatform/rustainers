use std::fmt::{self, Display};
use std::fs::Permissions;
use std::mem;
use std::os::unix::prelude::PermissionsExt;
use std::sync::Arc;

use crate::compose::{
    ComposeError, RunnableComposeContainers, RunnableComposeContainersBuilder, TemporaryDirectory,
    TemporaryFile, ToRunnableComposeContainers,
};
use crate::{ExposedPort, Port, PortError, SharedExposedPort, WaitStrategy};

const KAFKA_SERVICE: &str = "kafka";
const KAFKA_PORT: Port = Port(9092);

const SCHEMA_REGISTRY_SERVICE: &str = "schema-registry";
const SCHEMA_REGISTRY_PORT: Port = Port(8081);

/// A docker compose with a single node Kafka with kraft (aka. without zookeeper)
/// and a schema registry
#[derive(Debug, Clone)]
pub struct KafkaSchemaRegistry {
    temp_dir: TemporaryDirectory,
    kafka_port: SharedExposedPort,
    schema_registry_port: SharedExposedPort,
}

impl Display for KafkaSchemaRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Kafka + schema registry")
    }
}

impl KafkaSchemaRegistry {
    /// Create a [`KafkaSchemaRegistry`]
    ///
    /// # Errors
    ///
    /// Fail if we cannot create the temporary directory
    pub async fn build_single_kraft() -> Result<Self, ComposeError> {
        let kafka_port = ExposedPort::shared(KAFKA_PORT);
        let schema_registry_port = ExposedPort::shared(SCHEMA_REGISTRY_PORT);
        let temp_dir = TemporaryDirectory::with_files(
            "kafka_schema_registry",
            [
                // TODO provide a macro rules for simple cases
                TemporaryFile::builder()
                    .with_path("docker-compose.yaml")
                    .with_content(include_bytes!("./docker-compose.kraft.yaml"))
                    .build(),
                TemporaryFile::builder()
                    .with_path("kafka_update_run.sh")
                    .with_content(include_bytes!("./kafka_update_run.sh"))
                    .with_permissions(Permissions::from_mode(0o755)) // Execute
                    .build(),
            ],
        )
        .await?;

        Ok(Self {
            temp_dir,
            kafka_port,
            schema_registry_port,
        })
    }

    /// The Kafka broker address
    ///
    /// # Errors
    ///
    /// Fail if we cannot retrieve the Kafka host port
    pub async fn broker_address(&self) -> Result<String, PortError> {
        let p = self.kafka_port.lock().await;
        let port = p.host_port()?;
        mem::drop(p);
        let addr = format!("localhost:{port}");

        Ok(addr)
    }

    /// The schema registry endpoint
    ///
    /// # Errors
    ///
    /// Fail if we cannot retrieve the schema registry host port
    pub async fn schema_registry_endpoint(&self) -> Result<String, PortError> {
        let p = self.schema_registry_port.lock().await;
        let port = p.host_port()?;
        mem::drop(p);
        let addr = format!("http://localhost:{port}");

        Ok(addr)
    }
}

impl ToRunnableComposeContainers for KafkaSchemaRegistry {
    type AsPath = TemporaryDirectory;

    fn to_runnable(
        &self,
        builder: RunnableComposeContainersBuilder<Self::AsPath>,
    ) -> RunnableComposeContainers<Self::AsPath> {
        builder
            .with_compose_path(self.temp_dir.clone())
            .with_port_mappings([
                (KAFKA_SERVICE, Arc::clone(&self.kafka_port)),
                (
                    SCHEMA_REGISTRY_SERVICE,
                    Arc::clone(&self.schema_registry_port),
                ),
            ])
            .with_wait_strategies([
                (KAFKA_SERVICE, WaitStrategy::HealthCheck),
                (SCHEMA_REGISTRY_SERVICE, WaitStrategy::HealthCheck),
            ])
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn should_build_kafka_schema_registry() {
        _ = tracing_subscriber::fmt::try_init();

        let image = KafkaSchemaRegistry::build_single_kraft().await.unwrap();
        let dir = image.temp_dir.as_ref().to_path_buf();

        assert!(dir.join("docker-compose.yaml").exists());
        assert!(dir.join("kafka_update_run.sh").exists());
    }
}
