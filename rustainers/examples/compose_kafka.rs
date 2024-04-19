//! Example to show how we can use Kafka

use std::time::Duration;

use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::ClientConfig;
use tracing::{info, Level};

use rustainers::compose::images::KafkaSchemaRegistry;
use rustainers::runner::Runner;

mod common;
pub use self::common::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing(Level::INFO);

    let runner = Runner::auto()?;
    let image = KafkaSchemaRegistry::build_single_kraft().await?;

    let containers = runner.compose_start(image).await?;

    info!("Now I can use {containers}");
    do_something_with_kafka(&containers).await?;

    Ok(())
}

async fn do_something_with_kafka(image: &KafkaSchemaRegistry) -> anyhow::Result<()> {
    let topic = "plop";
    let broker_address = image.broker_address().await?;
    info!("Using kafka broker: {broker_address}");

    let schema_registry_url = image.schema_registry_endpoint().await?;
    info!("Using schema registry: {schema_registry_url}");

    let mut config = ClientConfig::new();
    config.set("bootstrap.servers", broker_address);
    config.set("message.timeout.ms", "5000");
    info!("Config {config:#?}");

    let producer: &FutureProducer = &config.create()?;
    let record = FutureRecord::to(topic).payload("plop").key(&());
    let sent = producer
        .send(record, Duration::from_secs(0))
        .await
        .map_err(|(err, _)| err)?;
    info!("✉️ {sent:?}");

    Ok(())
}
