use std::sync::Arc;
use std::time::Duration;

use proc_qq::re_exports::ricq;
use proc_qq::re_exports::ricq::version::ANDROID_PHONE;
use proc_qq::*;
use proc_qq_examples::hello_module;
use proc_qq_examples::init_tracing_subscriber;
use proc_qq_examples::result_handlers;
use proc_qq_examples::scheduler_handlers;

#[tokio::main]
async fn main() {
    init_tracing_subscriber();
    let qsign =
        ricq::qsign::QSignClient::new("url".to_owned(), "key ".to_owned(), Duration::from_secs(60))
            .expect("qsign client build err");
    let client = ClientBuilder::new()
        .authentication(Authentication::UinPasswordMd5(123456, [0; 16]))
        .show_slider_pop_menu_if_possible()
        .device(DeviceSource::JsonFile("device.json".to_owned()))
        .version(&ANDROID_PHONE)
        .qsign(Some(Arc::new(qsign)))
        .session_store(Box::new(FileSessionStore {
            path: "session.token".to_string(),
        }))
        .modules(vec![hello_module::module()])
        .result_handlers(vec![result_handlers::on_result {}.into()])
        .schedulers(vec![scheduler_handlers::scheduler()])
        .build()
        .await
        .unwrap();
    run_client(Arc::new(client)).await.unwrap();
}
