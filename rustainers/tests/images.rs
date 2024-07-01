#![allow(clippy::expect_used)]

use std::time::SystemTime;

use assert2::{check, let_assert};
use rstest::rstest;
use tokio::task::JoinSet;
use tracing::{debug, info};

use rustainers::images::{Minio, Mongo, Mosquitto, Postgres, Redis};
use rustainers::runner::{RunOption, Runner};
use rustainers::{ExposedPort, Port};

mod common;
pub use self::common::*;

#[rstest]
#[tokio::test]
async fn test_image_postgres(runner: &Runner) -> anyhow::Result<()> {
    let options = RunOption::builder().with_remove(true).build();
    let image = Postgres::default();
    let container = runner.start_with_options(image, options).await?;
    debug!("Started {container}");

    container.url().await?;
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_postgres_build_config(runner: &Runner) -> anyhow::Result<()> {
    let options = RunOption::builder().with_remove(true).build();
    let image = Postgres::default().with_port(ExposedPort::fixed(Port::new(5432), Port::new(5432)));
    let container = runner.start_with_options(image, options).await?;
    debug!("Started {container}");

    let result = container.config().await.expect("config");
    check!(result == "host=127.0.0.1 user=postgres password=passwd port=5432 dbname=postgres");

    let result = container.url().await.expect("url");
    check!(result == "postgresql://postgres:passwd@127.0.0.1:5432/postgres");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_image_minio(runner: &Runner) -> anyhow::Result<()> {
    let options = RunOption::builder().with_remove(true).build();
    let image = Minio::default();
    let container = runner.start_with_options(image, options).await?;
    debug!("Started {container}");

    container.endpoint().await?;
    container.console_endpoint().await?;
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_minio_endpoint(runner: &Runner) -> anyhow::Result<()> {
    let options = RunOption::builder().with_remove(true).build();
    let image = Minio::default().with_port(ExposedPort::fixed(Port::new(9000), Port::new(9124)));
    let container = runner.start_with_options(image, options).await?;
    debug!("Started {container}");

    let result = container.endpoint().await;
    let_assert!(Ok(endpoint) = result);
    check!(endpoint == "http://127.0.0.1:9124");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_image_redis(runner: &Runner) -> anyhow::Result<()> {
    let options = RunOption::builder().with_remove(true).build();
    let image = Redis::default();
    let container = runner.start_with_options(image, options).await?;
    debug!("Started {container}");

    container.endpoint().await?;
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_redis_endpoint(runner: &Runner) -> anyhow::Result<()> {
    let options = RunOption::builder().with_remove(true).build();
    let image = Redis::default().with_port(ExposedPort::fixed(Port::new(6379), Port::new(9125)));
    let container = runner.start_with_options(image, options).await?;
    debug!("Started {container}");

    let result = container.endpoint().await;
    let_assert!(Ok(endpoint) = result);
    check!(endpoint == "redis://127.0.0.1:9125");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_image_mongo(runner: &Runner) -> anyhow::Result<()> {
    let options = RunOption::builder().with_remove(true).build();
    let image = Mongo::default();
    let container = runner.start_with_options(image, options).await?;
    debug!("Started {container}");

    container.endpoint().await?;
    Ok(())
}
#[rstest]
#[tokio::test]
async fn test_mongo_endpoint(runner: &Runner) -> anyhow::Result<()> {
    let options = RunOption::builder().with_remove(true).build();
    let image = Mongo::default().with_port(ExposedPort::fixed(Port::new(27017), Port::new(9126)));
    let container = runner.start_with_options(image, options).await?;
    debug!("Started {container}");

    let result = container.endpoint().await;
    let_assert!(Ok(endpoint) = result);
    check!(endpoint == "mongodb://127.0.0.1:9126");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_run_in_multiple_tasks(runner: &Runner) -> anyhow::Result<()> {
    if let Runner::Docker(_) = &runner {
        // Work with docker, but fail with podman
        // FIXME find a solution
        return Ok(());
    }
    let start = SystemTime::now();
    let mut set = JoinSet::new();
    let size = 20;

    for id in 0..size {
        let img = Redis::default();
        let runner = runner.clone();
        set.spawn(async move {
            let container = runner.start(img).await.expect("container started");
            (id, container)
        });
    }

    // wait all
    let mut finished = vec![false; size];
    #[allow(clippy::indexing_slicing)]
    while let Some(Ok((id, container))) = set.join_next().await {
        info!("Container #{id} {container:#?}");
        finished[id] = true;
    }
    let duration = start.elapsed()?;
    for (id, value) in finished.iter().enumerate() {
        check!(*value == true, "Task #{id} not finished");
    }
    info!("Took {}s", duration.as_secs_f32());

    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_mosquitto_endpoint(runner: &Runner) -> anyhow::Result<()> {
    let options = RunOption::builder().with_remove(true).build();
    let image =
        Mosquitto::default().with_port(ExposedPort::fixed(Port::new(6379), Port::new(9127)));
    let container = runner.start_with_options(image, options).await?;
    debug!("Started {container}");

    let result = container.endpoint().await;
    let_assert!(Ok(endpoint) = result);
    check!(endpoint == "mqtt://127.0.0.1:9127");
    Ok(())
}
