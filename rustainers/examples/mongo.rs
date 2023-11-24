use std::time::Duration;

// use mongo::{Client, Commands};
use tracing::{info, Level};

use mongodb::bson::{doc, Document};
use mongodb::{options::ClientOptions, Client};
use rustainers::images::Mongo;
use rustainers::runner::{RunOption, Runner};
mod common;
pub use self::common::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing(Level::INFO);

    let runner = Runner::auto()?;
    let image = Mongo::default();
    let options = RunOption::builder()
        .with_remove(true)
        .with_wait_interval(Duration::from_millis(96))
        .build();

    let container = runner.start_with_options(image, options).await?;
    info!("Now I can use {container}");

    do_something_in_mongo(&container).await?;
    Ok(())
}

async fn do_something_in_mongo(mongo: &Mongo) -> anyhow::Result<()> {
    let endpoint = mongo.endpoint().await?;
    info!("Using Mongo at {endpoint}");

    let client_options = ClientOptions::parse(endpoint).await?;
    let client = Client::with_options(client_options)?;

    let db = client.database("plop");
    let collection = db.collection::<Document>("blob");

    let docs = vec![
        doc! { "title": "1984", "author": "George Orwell" },
        doc! { "title": "Animal Farm", "author": "George Orwell" },
        doc! { "title": "The Great Gatsby", "author": "F. Scott Fitzgerald" },
    ];

    collection.insert_many(docs, None).await?;

    let count = collection.count_documents(None, None).await?;

    println!("Number of documents in the collection: {count}");

    Ok(())
}
