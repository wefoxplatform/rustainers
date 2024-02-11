use std::time::SystemTime;

use assert2::check;
use rstest::rstest;
use tokio::task::JoinSet;
use tracing::debug;

use rustainers::images::{Minio, Mongo, Postgres, Redis};
use rustainers::runner::{RunOption, Runner};

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
        let r = runner.clone();
        set.spawn(async move {
            let container = r.start(img).await.unwrap();
            (id, container)
        });
    }

    // wait all
    let mut finished = vec![false; size];
    while let Some(Ok((id, container))) = set.join_next().await {
        println!("Container #{id} {container:#?}");
        finished[id] = true;
    }
    let duration = start.elapsed()?;
    for (id, v) in finished.iter().enumerate() {
        check!(*v == true, "Task #{id} not finished");
    }
    println!("Took {}s", duration.as_secs_f32());

    Ok(())
}
