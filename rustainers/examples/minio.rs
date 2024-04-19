//! Example to use `MinIO`

use std::time::Duration;

use bytes::Bytes;
use futures_util::StreamExt;
use object_store::aws::AmazonS3Builder;
use object_store::path::Path;
use object_store::ObjectStore;
use tracing::{info, Level};

use rustainers::images::Minio;
use rustainers::runner::{RunOption, Runner};

mod common;
pub use self::common::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing(Level::INFO);

    let runner = Runner::auto()?;
    let image = Minio::default();
    let options = RunOption::builder()
        .with_remove(true)
        .with_wait_interval(Duration::from_millis(300))
        .build();

    let container = runner.start_with_options(image, options).await?;
    info!("Now I can use {container}");

    let bucket_name = "plop-bucket";
    container.create_s3_bucket(bucket_name).await?;
    info!("Bucket {bucket_name} created");

    do_something_in_minio(&container, bucket_name).await?;

    Ok(())
}

async fn do_something_in_minio(minio: &Minio, bucket_name: &str) -> anyhow::Result<()> {
    let endpoint = minio.endpoint().await?;
    info!("Using MinIO at {endpoint}");
    let s3 = AmazonS3Builder::from_env()
        .with_region(minio.region())
        .with_endpoint(endpoint)
        .with_bucket_name(bucket_name)
        .with_allow_http(true)
        .with_access_key_id(minio.access_key_id())
        .with_secret_access_key(minio.secret_access_key())
        .build()?;

    // Store an object
    s3.put(&Path::from("plop.txt"), Bytes::from_static(b"plop"))
        .await?;

    // list objects
    let mut stream = s3.list(None);
    while let Some(res) = stream.next().await {
        let obj = res?;
        info!("🎉 file: {obj:?}");
    }

    Ok(())
}
