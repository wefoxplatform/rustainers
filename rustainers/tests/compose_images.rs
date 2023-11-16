mod common;
pub use self::common::*;

#[cfg(feature = "very-long-tests")]
mod kafka {

    use rstest::rstest;
    use tracing::debug;

    use rustainers::compose::images::KafkaSchemaRegistry;
    use rustainers::runner::Runner;

    pub use super::*;

    #[rstest]
    #[tokio::test]
    async fn test_kafka_schema_registry_image(runner: &Runner) -> anyhow::Result<()> {
        let image = KafkaSchemaRegistry::build_single_kraft().await?;
        debug!("Image {image}");

        let containers = runner.compose_start(image).await?;
        debug!("Started {containers}");
        containers.broker_address().await?;
        containers.schema_registry_endpoint().await?;

        Ok(())
    }
}

mod redpanda {
    use rstest::rstest;
    use tracing::debug;

    use rustainers::compose::images::Redpanda;
    use rustainers::runner::Runner;

    pub use super::*;

    #[rstest]
    #[tokio::test]
    async fn test_redpanda_schema_registry_image(runner: &Runner) -> anyhow::Result<()> {
        let image = Redpanda::build_single().await?;
        debug!("Image {image}");

        let containers = runner.compose_start(image).await?;
        debug!("Started {containers}");
        containers.broker_address().await?;
        containers.schema_registry_endpoint().await?;

        Ok(())
    }
}
