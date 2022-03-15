use tracing::Level;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use proc_qq::re_exports::rs_qq::version::ANDROID_WATCH;
use proc_qq::Authentication::QRCode;
use proc_qq::{ClientBuilder, DeviceSource};

mod modules;
mod tools;

#[tokio::main]
async fn main() {
    init_tracing_subscriber();
    ClientBuilder::new()
        .device(DeviceSource::JsonFile("device.json".to_owned()))
        .version(&ANDROID_WATCH)
        .priority_session("session.token")
        .authentication(QRCode)
        .build(modules::all_modules())
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
                .with_target("proc_qq_template", Level::DEBUG),
        )
        .init();
}
