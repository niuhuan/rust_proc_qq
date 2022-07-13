use std::sync::Arc;

use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use proc_qq::re_exports::ricq::version::ANDROID_WATCH;
use proc_qq::Authentication::{QRCode, UinPassword};
#[cfg(any(target_os = "windows"))]
use proc_qq::ShowSlider;
use proc_qq::{ClientBuilder, DeviceSource, ShowQR};

use crate::hello_module;
use crate::result_handlers;

#[tokio::test]
async fn test_qr_login() {
    init_tracing_subscriber();
    ClientBuilder::new()
        .device(DeviceSource::JsonFile("device.json".to_owned()))
        .version(&ANDROID_WATCH)
        // .priority_session("session.token")
        .authentication(QRCode)
        .modules(vec![hello_module::module()])
        .show_rq(Some(ShowQR::OpenBySystem))
        .build()
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
    let cb = ClientBuilder::new()
        .device(DeviceSource::JsonFile("device.json".to_owned()))
        .version(&ANDROID_WATCH)
        // .priority_session("session.token")
        ;
    // windows桌面gui滑块
    #[cfg(any(target_os = "windows"))]
    let cb = cb.show_slider(Some(ShowSlider::PopWindow)); // 从桌面弹出窗口进行滑块, 只支持windows
    cb.authentication(UinPassword(123456, "password".to_owned()))
        .modules(Arc::new(vec![hello_module::module()]))
        .result_handlers(vec![result_handlers::on_result {}.into()])
        .build()
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
                .with_target("ricq", Level::DEBUG)
                .with_target("proc_qq", Level::DEBUG)
                // 如果需搬运, 这里换成自己的项目名
                .with_target("proc_qq_examples", Level::DEBUG),
        )
        .init();
}
