use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use proc_qq::re_export::rs_qq::handler::DefaultHandler;
use proc_qq::Authentication::{QRCode, UinPassword};
use proc_qq::ClientBuilder;

#[tokio::test]
async fn test_qr_login() {
    init_tracing_subscriber();
    ClientBuilder::new()
        .priority_session("session.token")
        .authentication(QRCode)
        .build(DefaultHandler {})
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
        .priority_session("session.token")
        .authentication(UinPassword(123456, "password".to_owned()))
        .build(DefaultHandler {})
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
                .with_target("proc_qq_test", Level::DEBUG),
        )
        .init();
}
