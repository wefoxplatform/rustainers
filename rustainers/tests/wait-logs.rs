//! Tests for log waits.

use std::time::{Duration, Instant};

use assert2::check;
use rstest::rstest;
use ulid::Ulid;

use rustainers::images::GenericImage;
use rustainers::runner::Runner;
use rustainers::{ImageName, WaitStrategy};

mod common;
pub use self::common::*;

#[rstest]
#[tokio::test]
async fn should_wait_stdout_contains(runner: &Runner) -> anyhow::Result<()> {
    let id = Ulid::new().to_string();

    let mut image = GenericImage::new(ImageName::new("alpine"));
    image.set_wait_strategy(WaitStrategy::stdout_contains(&id));
    image.set_command(["sh", "-c", &format!("sleep 1 && echo {id}")]);

    let start = Instant::now();
    let _container = runner.start(image).await?;
    let duration = start.elapsed();
    check!(duration > Duration::from_secs(1));

    Ok(())
}

#[cfg(feature = "regex")]
#[rstest]
#[tokio::test]
async fn should_wait_stderr_match(runner: &Runner) -> anyhow::Result<()> {
    use regex::Regex;

    let re = Regex::new("OK to (?<task>.+)")?;

    let mut image = GenericImage::new(ImageName::new("alpine"));
    image.set_wait_strategy(WaitStrategy::stderr_match(re));
    image.set_command(["sh", "-c", "sleep 1 && logger -s 'OK to continue'"]);

    let start = Instant::now();
    let _container = runner.start(image).await?;
    let duration = start.elapsed();
    check!(duration > Duration::from_secs(1));

    Ok(())
}
