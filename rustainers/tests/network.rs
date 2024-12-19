//! Network-related tests.

use assert2::let_assert;
use rstest::rstest;
use rustainers::runner::{RunOption, Runner};
use ulid::Ulid;

mod common;
pub use self::common::*;
use self::images::InternalWebServer;
use crate::images::Curl;

#[rstest]
#[tokio::test]
async fn should_work_with_network(runner: &Runner) -> anyhow::Result<()> {
    let id = Ulid::new();
    // Create network
    let name = format!("my_network_{id}",);
    let network = runner.create_network(&name).await?;

    // Container A inside network
    let options = RunOption::builder()
        .with_name(format!("web-server_{id}"))
        .with_remove(true)
        .with_network(network.clone()) // network
        .build();
    let _container = runner
        .start_with_options(InternalWebServer, options)
        .await?;

    let url = format!("http://web-server_{id}:80");
    let result = images::curl(
        runner,
        url,
        RunOption::builder()
            .with_name(format!("curl_{id}"))
            .with_network(network)
            .build(),
    )
    .await;

    let_assert!(Ok(()) = result);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn should_work_with_network_ip(runner: &Runner) -> anyhow::Result<()> {
    let id = Ulid::new();
    // Create network
    let name = format!("my_network_{id}",);
    let network = runner.create_network(&name).await?;

    // Container A inside network
    let options = RunOption::builder()
        .with_name(format!("web-server_{id}"))
        .with_remove(true)
        .with_network(network.clone()) // network
        .build();
    let container = runner
        .start_with_options(InternalWebServer, options)
        .await?;

    let network_ip = runner.network_ip(&container, &network).await?;
    let url = format!("http://{network_ip}:80");
    let result = images::curl(
        runner,
        url,
        RunOption::builder()
            .with_name(format!("curl_{id}"))
            .with_network(network)
            .build(),
    )
    .await;

    let_assert!(Ok(()) = result);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn should_work_dind(runner: &Runner) -> anyhow::Result<()> {
    let id = Ulid::new();

    // Container A inside network
    let server_options = RunOption::builder()
        .with_name(format!("web-server_{id}"))
        .with_remove(true)
        .build();

    let _ = runner
        .start_with_options(InternalWebServer, server_options)
        .await?;

    let_assert!(Ok(host) = runner.container_host_ip().await);
    let client_options = RunOption::builder()
        .with_name(format!("client_{id}"))
        .build();
    let url = format!("http://{host}:80");
    let image = Curl { url };
    let _ = runner.start_with_options(image, client_options).await?;

    Ok(())
}

#[rstest]
#[tokio::test]
async fn should_not_work_without_network(runner: &Runner) -> anyhow::Result<()> {
    let id = Ulid::new();
    // Create network
    let name = format!("my_network_{id}",);
    let network = runner.create_network(&name).await?;

    // Container A inside network
    let options = RunOption::builder()
        .with_name(format!("web-server_{id}"))
        .with_remove(true)
        .with_network(network.clone()) // network
        .build();
    let _container = runner
        .start_with_options(InternalWebServer, options)
        .await?;

    let url = format!("http://web-server_{id}:80");
    let result = images::curl(
        runner,
        url,
        RunOption::builder()
            .with_name(format!("curl_{id}"))
            // Not the same network
            .build(),
    )
    .await;

    let_assert!(Ok(()) = result);

    Ok(())
}
