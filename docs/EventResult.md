事件结果
======

```rust
use proc_qq::result;
use proc_qq::EventResult;

#[result]
async fn on_result(result: &EventResult) -> anyhow::Result<bool> {
    match result {
        EventResult::Process(info) => {
            tracing::info!("{} : {} : 处理了一条消息", info.module_id, info.handle_name);
        }
        EventResult::Exception(info, err) => {
            tracing::info!(
                "{} : {} : 遇到了错误 : {}",
                info.module_id,
                info.handle_name,
                err
            );
        }
    }
    Ok(false)
}
```

```rust
ClientBuilder::new()
.modules(vec![hello_module::module()])
.result_handlers(vec![result_handlers::on_result {}.into()])
.build()
```
