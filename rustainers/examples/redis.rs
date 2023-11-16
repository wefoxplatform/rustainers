use std::time::Duration;

use redis::{Client, Commands};
use tracing::{info, Level};

use rustainers::images::Redis;
use rustainers::runner::{RunOption, Runner};

mod common;
pub use self::common::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing(Level::INFO);

    let runner = Runner::auto()?;
    let image = Redis::default();
    let options = RunOption::builder()
        .with_remove(true)
        .with_wait_interval(Duration::from_millis(96))
        .build();

    let container = runner.start_with_options(image, options).await?;
    info!("Now I can use {container}");

    do_something_in_redis(&container).await?;

    Ok(())
}

async fn do_something_in_redis(redis: &Redis) -> anyhow::Result<()> {
    let endpoint = redis.endpoint().await?;
    info!("Using Redis at {endpoint}");
    let client = Client::open(endpoint)?;
    let mut con = client.get_connection()?;
    let key = "plop";
    // throw away the result, just make sure it does not fail
    con.set(key, "plop-123")?;
    // read back the key and return it.  Because the return value
    // from the function is a result for integer this will automatically
    // convert into one.
    let result = con.get::<_, String>(&key)?;
    println!("Result: {result}");

    Ok(())
}
