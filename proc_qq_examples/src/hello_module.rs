use proc_qq::re_export::rs_qq::client::event::GroupMessageEvent;
use proc_qq::{
    event, module, MessageChainParseTrait, MessageContentTrait, MessageEvent,
    MessageSendToSourceTrait, Module,
};

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
    } else {
        Ok(false)
    }
}

#[event]
async fn group_hello(_: &GroupMessageEvent) -> anyhow::Result<bool> {
    Ok(false)
}

pub(crate) fn module() -> Module {
    module!("hello", "你好", print, group_hello)
}