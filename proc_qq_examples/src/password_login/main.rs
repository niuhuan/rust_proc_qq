use proc_qq::re_exports::ricq::version::ANDROID_WATCH;
use proc_qq::*;

use proc_qq_examples::hello_module;
use proc_qq_examples::init_tracing_subscriber;
use proc_qq_examples::result_handlers;

#[tokio::main]
async fn main() {
    init_tracing_subscriber();
    ClientBuilder::new()
        .authentication(Authentication::UinPasswordMd5(123456, [0; 16]))
        .show_slider_pop_menu_if_possible()
        .device(DeviceSource::JsonFile("device.json".to_owned()))
        .version(&ANDROID_WATCH)
        .modules(vec![hello_module::module()])
        .result_handlers(vec![result_handlers::on_result {}.into()])
        .build()
        .await
        .unwrap()
        .start()
        .await
        .unwrap()
        .unwrap();
}
