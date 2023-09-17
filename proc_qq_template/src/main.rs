extern crate core;

use crate::config::load_config;
use crate::database::mongo::init_mongo;
use crate::database::redis::init_redis;
use proc_qq::re_exports::ricq;
use proc_qq::re_exports::ricq::version::ANDROID_WATCH;
use proc_qq::Authentication::UinPasswordMd5;
use proc_qq::{run_client, ClientBuilder, DeviceSource, FileSessionStore};
use std::sync::Arc;
use std::time::Duration;
use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod config;
mod database;
mod modules;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing_subscriber();
    let config = load_config().await?;
    init_redis(&config.redis).await?;
    init_mongo(&config.mongo).await?;
    modules::init_modules().await?;
    let password_vec = hex::decode(config.account.password_md5)?;
    let mut password = [0 as u8; 16];
    password[..16].clone_from_slice(password_vec.as_slice());
    let qsign =
        ricq::qsign::QSignClient::new("url".to_owned(), "key ".to_owned(), Duration::from_secs(60))
            .expect("qsign client build err");
    let client = ClientBuilder::new()
        .device(DeviceSource::JsonFile("device.json".to_owned()))
        .version(&ANDROID_WATCH)
        .session_store(FileSessionStore::boxed("session.token"))
        .authentication(UinPasswordMd5(config.account.uin, password))
        .qsign(Some(Arc::new(qsign)))
        .show_slider_pop_menu_if_possible()
        .modules(modules::all_modules())
        .build()
        .await
        .unwrap();
    // 可以做一些定时任务, rq_client在一开始可能没有登录好
    let client = Arc::new(client);
    let copy = client.clone();
    tokio::spawn(async move {
        println!("{}", copy.rq_client.start_time);
    });
    run_client(client).await?;
    Ok(())
}

fn init_tracing_subscriber() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .without_time(),
        )
        .with(
            tracing_subscriber::filter::Targets::new()
                .with_target("ricq", Level::DEBUG)
                .with_target("proc_qq", Level::DEBUG)
                .with_target("proc_qq_template", Level::DEBUG),
        )
        .init();
}
