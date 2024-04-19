use std::collections::HashMap;

use assert2::check;
use rstest::rstest;
use ulid::Ulid;

use rustainers::images::Alpine;
use rustainers::runner::{RunOption, Runner};

mod common;
pub use self::common::*;

#[rstest]
#[tokio::test]
#[allow(clippy::indexing_slicing)]
async fn should_work_with_env(runner: &Runner) -> anyhow::Result<()> {
    let data1 = Ulid::new();
    let data2 = "With space   ";
    let data3 = "with = plop";

    let options = RunOption::builder()
        .with_remove(true)
        .with_env([
            ("TEST_ENV_DATA1", data1.to_string()),
            ("TEST_ENV_DATA2", data2.to_string()),
            ("TEST_ENV_DATA3", data3.to_string()),
        ])
        .build();
    let container = runner.start_with_options(Alpine, options).await?;

    let result = runner.exec(&container, ["env"]).await?;
    let vars = result
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.split_once('='))
        .collect::<HashMap<_, _>>();

    check!(vars["TEST_ENV_DATA1"] == data1.to_string());
    check!(vars["TEST_ENV_DATA2"] == data2);
    check!(vars["TEST_ENV_DATA3"] == data3);

    Ok(())
}
