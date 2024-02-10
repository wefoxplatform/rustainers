use assert2::let_assert;
use rstest::rstest;
use ulid::Ulid;

use rustainers::images::Alpine;
use rustainers::runner::{RunOption, Runner};
use rustainers::Volume;

mod common;
pub use self::common::*;

#[rstest]
#[tokio::test]
async fn should_copy_dir_with_volume(runner: &Runner) -> anyhow::Result<()> {
    let id = Ulid::new();

    // Create a container volume
    let name = format!("volume_{id}");
    let volume_name = runner.create_volume(&name).await?;

    // Copy page to volume
    runner
        .copy_to_volume(volume_name.clone(), "tests/assets")
        .await?;

    let dest = format!("/plop/tmp{id}");
    let mut volume = Volume::container_volume(volume_name.clone(), &dest);
    volume.read_only();

    // Bind mount volume
    let options = RunOption::builder()
        .with_remove(true)
        .with_volumes([volume])
        .build();
    let container = runner.start_with_options(Alpine, options).await?;

    let target = format!("{dest}/assets");
    let result = runner.exec(&container, ["ls", &target]).await;
    let_assert!(Ok(ls) = result);
    let files = ls.lines().collect::<Vec<_>>();
    assert!(files.contains(&"index.html"));

    Ok(())
}
