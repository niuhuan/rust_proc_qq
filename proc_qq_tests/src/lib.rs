use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use proc_qq::Authentication::{QRCode, UinPassword};
use proc_qq::ClientBuilder;

// use proc_qq::event;
// use proc_qq::re_export::rs_qq::client::event::GroupMessageEvent;

// #[event]
// async fn a(event: &GroupMessageEvent) {}

#[tokio::test]
async fn test_qr_login() {
    init_tracing_subscriber();
    ClientBuilder::new()
        .priority_session("session.token")
        .authentication(QRCode)
        .build(vec![])
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
        .build(vec![])
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
