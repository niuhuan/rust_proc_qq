use proc_qq::re_exports::ricq::version::ANDROID_WATCH;
use proc_qq::*;
use std::time::Duration;

use proc_qq_examples::hello_module;
use proc_qq_examples::init_tracing_subscriber;
use proc_qq_examples::result_handlers;

fn main() {
    init_tracing_subscriber();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_keep_alive(Duration::new(60, 0))
        .worker_threads(30)
        .max_blocking_threads(30)
        .build()
        .unwrap()
        .block_on(async {
            ClientBuilder::new()
                .authentication(Authentication::UinPasswordMd5(123456, [0; 16]))
                .show_slider(ShowSlider::PopWindow)
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
                .unwrap()
        });
}
