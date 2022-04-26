use std::sync::Arc;
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use proc_qq::re_exports::rs_qq::version::ANDROID_WATCH;
use proc_qq::Authentication::{QRCode, UinPassword};
use proc_qq::{ClientBuilder, DeviceSource};

mod hello_module;

#[tokio::test]
async fn test_qr_login() {
    init_tracing_subscriber();
    ClientBuilder::new()
        .device(DeviceSource::JsonFile("device.json".to_owned()))
        .version(&ANDROID_WATCH)
        .priority_session("session.token")
        .authentication(QRCode)
        .build(vec![hello_module::module()])
        .await
        .unwrap()
        .start()
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn test_password_login() {
    init_tracing_subscriber();
    ClientBuilder::new()
        .device(DeviceSource::JsonFile("device.json".to_owned()))
        .version(&ANDROID_WATCH)
        .priority_session("session.token")
        .authentication(UinPassword(123456, "password".to_owned()))
        .build(Arc::new(vec![hello_module::module()]))
        .await
        .unwrap()
        .start()
        .await
        .unwrap()
        .unwrap();
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
                // 如果需搬运, 这里换成自己的项目名
                .with_target("proc_qq_examples", Level::DEBUG),
        )
        .init();
}
