use assert2::let_assert;
use rstest::rstest;

use rustainers::runner::{RunOption, Runner};
use rustainers::{
    ContainerStatus, ImageName, RunnableContainer, RunnableContainerBuilder, ToRunnableContainer,
};

mod common;
pub use self::common::*;

#[derive(Debug, Clone, Copy)]
struct HelloWorld;

impl ToRunnableContainer for HelloWorld {
    fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
        builder
            .with_image(ImageName::new("hello-world"))
            .with_wait_strategy(ContainerStatus::Exited)
            .build()
    }
}

#[rstest]
#[tokio::test]
async fn should_run_hello_world(runner: &Runner) {
    _ = tracing_subscriber::fmt::try_init();

    let result = runner
        .start_with_options(HelloWorld, RunOption::default())
        .await;
    if let Err(e) = &result {
        eprintln!("{e}");
    }
    let_assert!(Ok(_) = result);
}
