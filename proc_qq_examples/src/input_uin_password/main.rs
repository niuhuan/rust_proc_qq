use std::io::{stdin, stdout, Write};
use std::sync::Arc;

use proc_qq::re_exports::async_trait::async_trait;
use proc_qq::re_exports::ricq::version::ANDROID_PHONE;
use proc_qq::*;
use proc_qq_examples::init_tracing_subscriber;
use proc_qq_examples::result_handlers;
use proc_qq_examples::{hello_module, scheduler_handlers};

#[tokio::main]
async fn main() {
    init_tracing_subscriber();
    let client = ClientBuilder::new()
        .authentication(Authentication::CustomUinPassword(Arc::new(Box::new(
            InputUinPassword {},
        ))))
        .show_slider_pop_menu_if_possible()
        .device(DeviceSource::JsonFile("device.json".to_owned()))
        .version(&ANDROID_PHONE)
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

struct InputUinPassword;

#[async_trait]
impl CustomUinPassword for InputUinPassword {
    async fn input_uin(&self) -> anyhow::Result<i64> {
        Ok(input("请输入账号")?.parse()?)
    }

    async fn input_password(&self) -> anyhow::Result<String> {
        input("请输入密码")
    }
}

fn input(tips: &str) -> anyhow::Result<String> {
    let mut s = String::new();
    print!("{}: ", tips);
    let _ = stdout().flush();
    stdin().read_line(&mut s)?;
    if let Some('\n') = s.chars().next_back() {
        s.pop();
    }
    if let Some('\r') = s.chars().next_back() {
        s.pop();
    }
    Ok(s)
}
