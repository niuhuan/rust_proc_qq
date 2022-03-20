use crate::config::Mongo;
use anyhow::Context;
use mongodb::options::ClientOptions;
use mongodb::{Client, Collection, Database};
use once_cell::sync::OnceCell;

static DATABASE: OnceCell<String> = OnceCell::new();
static CLIENT: OnceCell<Client> = OnceCell::new();

pub(crate) async fn init_mongo(mongo: &Mongo) -> anyhow::Result<()> {
    DATABASE.set(mongo.database.clone()).unwrap();
    let client_options = ClientOptions::parse(format!(
        "mongodb://{}:{}/{}?w=majority",
        mongo.host, mongo.port, mongo.database
    ))
    .await?;
    let client = Client::with_options(client_options)?;
    client
        .list_database_names(None, None)
        .await
        .with_context(|| "mongo连接失败")?;
    CLIENT.set(client).unwrap();
    tracing::info!("mongo连接成功");
    Ok(())
}

#[allow(dead_code)]
pub(crate) async fn db() -> Database {
    CLIENT.get().unwrap().default_database().unwrap()
}

#[allow(dead_code)]
pub(crate) async fn collection<T>(collection_name: &str) -> Collection<T> {
    CLIENT
        .get()
        .unwrap()
        .default_database()
        .unwrap()
        .collection(collection_name)
}
