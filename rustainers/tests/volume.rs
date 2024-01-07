use assert2::{check, let_assert};
use rstest::rstest;
use rustainers::Volume;
use ulid::Ulid;

use rustainers::compose::{TemporaryDirectory, TemporaryFile};
use rustainers::runner::{RunOption, Runner};

mod common;
use crate::images::copy_file_to_volume;

pub use self::common::*;
use self::images::WebServer;

#[rstest]
#[tokio::test]
async fn should_work_with_volume_mount_bind(runner: &Runner) -> anyhow::Result<()> {
    let id = Ulid::new();

    // Create host temp.directory
    // XXX should use host_dir reference to avoid drop too early
    let name = format!("volume_{id}");
    let page = include_str!("assets/index.html");
    let host_dir = TemporaryDirectory::with_files(
        &name,
        [TemporaryFile::builder()
            .with_path("index.html")
            .with_content(page)
            .build()],
    )
    .await?;

    // Bind mount volume
    let options = RunOption::builder()
        .with_remove(true)
        .with_volumes([(&host_dir, WebServer::STATIC_HTML)])
        .build();
    let image = WebServer::default();
    let container = runner.start_with_options(image, options).await?;

    let result = container.get("/index.html").await;
    let_assert!(Ok(html) = result);
    check!(html == page);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn should_work_with_volume(runner: &Runner) -> anyhow::Result<()> {
    let id = Ulid::new();

    // Create a container volume
    let name = format!("volume_{id}");
    let volume_name = runner.create_volume(&name).await?;

    // Copy page to volume
    copy_file_to_volume(runner, volume_name.clone(), "tests/assets/index.html").await?;

    let mut volume = Volume::container_volume(volume_name.clone(), WebServer::STATIC_HTML);
    volume.read_only();

    // Bind mount volume
    let options = RunOption::builder()
        .with_remove(true)
        .with_volumes([volume])
        .build();
    let image = WebServer::default();
    let container = runner.start_with_options(image, options).await?;

    let result = container.get("/index.html").await;
    let_assert!(Ok(html) = result);
    let page = include_str!("assets/index.html");
    check!(html == page);

    Ok(())
}
