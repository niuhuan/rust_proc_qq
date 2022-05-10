PROC_QQ
=======

- Rust语言的QQ机器人框架.
- 开箱即用, 操作简单, 代码极简

相关链接

- [PROC_QQ](https://github.com/niuhuan/rust_proc_qq)
- [RICQ](https://github.com/lz1998/ricq)

Example

```rust

/// 事件处理器
#[event]
async fn print(event: &MessageEvent) -> anyhow::Result<bool> {
    let content = event.message_content();
    if content.eq("你好") {
        event
            .send_message_to_source("世界".parse_message_chain())
            .await?;
        Ok(true)
    } else if content.eq("RC") {
        event
            .send_message_to_source("NB".parse_message_chain())
            .await?;
        Ok(true)
    } else if content.eq("EX") {
        Err(anyhow::Error::msg("Text exception"))
    } else {
        Ok(false)
    }
}

/// 封装模块
pub(crate) fn module() -> Module {
    module!("hello", "你好", print)
}

/// 调用
#[tokio::test]
async fn test_qr_login() {
    init_tracing_subscriber();
    ClientBuilder::new()
        .device(DeviceSource::JsonFile("device.json".to_owned()))
        .version(&ANDROID_WATCH)
        .authentication(QRCode)
        .modules(vec![hello_module::module()])
        .build()
        .await
        .unwrap()
        .start()
        .await
        .unwrap()
        .unwrap();
}

```

RC -> 回复:"NB"
你好 -> 回复:"世界"

