pub use proc_qq::re_exports::async_trait::async_trait;
use proc_qq::{
    event, module, MessageChainParseTrait, MessageEvent, MessageSendToSourceTrait, Module,
};

#[event(eq = "你很好")]
async fn handle(event: &MessageEvent) -> anyhow::Result<bool> {
    event
        .send_message_to_source("你也很好".parse_message_chain())
        .await?;
    Ok(true)
}

pub fn module() -> Module {
    module!("hello2", "你好2", handle,)
}
