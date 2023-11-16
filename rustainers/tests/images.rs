use std::fmt::Debug;
use std::time::SystemTime;

use assert2::check;
use rstest::rstest;
use tokio::task::JoinSet;
use tracing::debug;

use rustainers::images::{Minio, Postgres, Redis};
use rustainers::runner::{RunOption, Runner};
use rustainers::ToRunnableContainer;

mod common;
pub use self::common::*;

#[rstest]
#[case::pg(Postgres::default())]
#[case::minio(Minio::default())]
#[case::redis(Redis::default())]
#[tokio::test]
async fn test_image(
    runner: &Runner,
    #[case] image: impl ToRunnableContainer + Debug,
) -> anyhow::Result<()> {
    let options = RunOption::builder().with_remove(true).build();
    debug!("Image {image:?}");

    let container = runner.start_with_options(image, options).await?;
    debug!("Started {container}");

    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_double_run(runner: &Runner, redis: &Redis) -> anyhow::Result<()> {
    let container = runner.start(redis.clone()).await?;
    println!("Container {container}");

    let container2 = runner.start(redis.clone()).await?;
    println!("Container2 {container2}");

    check!(
        container.id() != container2.id(),
        "Should create two containers"
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_reuse(runner: &Runner, redis: &Redis) -> anyhow::Result<()> {
    let option = RunOption::builder().with_name("my-redis").build();

    let container = runner
        .start_with_options(redis.clone(), option.clone())
        .await?;
    println!("Container {container}");

    let container2 = runner.start_with_options(redis.clone(), option).await?;
    println!("Container2 {container2}");

    check!(
        container.id() == container2.id(),
        "Should reuse the same container"
    );

    Ok(())
}

#[rstest]
#[tokio::test]
#[ignore = "work with docker, but fail with podman"] // FIXME find a solution
async fn test_run_in_multiple_tasks(runner: &Runner, redis: &Redis) -> anyhow::Result<()> {
    let start = SystemTime::now();
    let mut set = JoinSet::new();
    let size = 20;

    for id in 0..size {
        let img = redis.clone();
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
