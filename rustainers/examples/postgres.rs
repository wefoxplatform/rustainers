//! Example to use Postgres

use core::panic;
use std::time::Duration;

use tokio_postgres::NoTls;
use tracing::{info, warn, Level};

use rustainers::images::Postgres;
use rustainers::runner::{RunOption, Runner};

mod common;
pub use self::common::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing(Level::INFO);

    let runner = Runner::auto()?;
    let image = Postgres::default().with_db("plop");
    let options = RunOption::builder()
        .with_remove(true)
        .with_wait_interval(Duration::from_millis(300))
        .build();

    let container = runner.start_with_options(image, options).await?;
    info!("Now I can use {container}");
    do_something_in_postgres(&container).await?;

    Ok(())
}

async fn do_something_in_postgres(pg: &Postgres) -> anyhow::Result<()> {
    let config = pg.config().await?;

    // Connect to the database.
    let (client, connection) = tokio_postgres::connect(&config, NoTls).await?;

    // The connection object performs the actual communication with the database,
    // so spawn it off to run on its own.
    tokio::spawn(async move {
        if let Err(err) = connection.await {
            warn!("connection error: {err}");
        }
    });

    // Now we can execute a simple statement that just returns its parameter.
    let rows = client.query("SELECT $1::TEXT", &[&"hello world"]).await?;

    // And then check that we got back the same string we sent over.
    let Some(row) = rows.first() else {
        panic!("Oops, expected one row");
    };
    let value: &str = row.get(0);
    info!("🎉 Result: {value}");
    assert_eq!(value, "hello world");

    Ok(())
}
