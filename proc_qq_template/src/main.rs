use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::config::load_config;
use crate::database::mongo::init_mongo;
use crate::database::redis::init_redis;
use proc_qq::re_exports::rs_qq::version::ANDROID_WATCH;
use proc_qq::Authentication::UinPasswordMd5;
use proc_qq::{ClientBuilder, DeviceSource};

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
    let password_vec = hex::decode(config.account.password_md5)?;
    let mut password = [0 as u8; 16];
    password[..16].clone_from_slice(password_vec.as_slice());
    ClientBuilder::new()
        .device(DeviceSource::JsonFile("device.json".to_owned()))
        .version(&ANDROID_WATCH)
        .priority_session("session.token")
        .authentication(UinPasswordMd5(config.account.uin, password))
        .build(modules::all_modules())
        .await
        .unwrap()
        .start()
        .await
        .unwrap()
        .unwrap();
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
                .with_target("rs_qq", Level::DEBUG)
                .with_target("proc_qq", Level::DEBUG)
                .with_target("proc_qq_template", Level::DEBUG),
        )
        .init();
}
