//! Tests for waits.

mod common;
use assert2::let_assert;
use rstest::rstest;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use rustainers::runner::Runner;

pub use self::common::*;
use self::images::{Netcat, WebServer};

#[rstest]
#[tokio::test]
async fn should_wait_http(runner: &Runner) -> anyhow::Result<()> {
    let container = runner.start(WebServer::default()).await?;

    let result = container.get("/index.html").await;
    let_assert!(Ok(_) = result);

    Ok(())
}

#[rstest]
#[tokio::test]
async fn should_wait_scan_port(runner: &Runner) -> anyhow::Result<()> {
    let image = Netcat::default();
    let container = runner.start(image).await?;

    let addr = container.addr().await?;
    let mut stream = TcpStream::connect(addr).await?;
    stream.write_all(b"ping").await?;

    Ok(())
}
