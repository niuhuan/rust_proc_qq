use proc_qq::re_exports::ricq::version::ANDROID_WATCH;
use proc_qq::*;
use std::sync::Arc;

use proc_qq_examples::hello_module;
use proc_qq_examples::init_tracing_subscriber;
use proc_qq_examples::result_handlers;

#[tokio::main]
async fn main() {
    init_tracing_subscriber();
    let client = ClientBuilder::new()
        .authentication(Authentication::QRCode)
        .show_rq(ShowQR::OpenBySystem)
        .device(DeviceSource::JsonFile("device.json".to_owned()))
        .version(&ANDROID_WATCH)
        .session_store(FileSessionStore::boxed("session.token"))
        .modules(vec![hello_module::module()])
        .result_handlers(vec![result_handlers::on_result {}.into()])
        .build()
        .await
        .unwrap();
    run_client(Arc::new(client)).await.unwrap();
}
