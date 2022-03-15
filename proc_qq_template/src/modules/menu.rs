use crate::modules::all_modules;
use proc_qq::{
    event, module, MessageChainParseTrait, MessageContentTrait, MessageEvent,
    MessageSendToSourceTrait, Module,
};

static ID: &'static str = "menu";
static NAME: &'static str = "菜单";

pub fn module() -> Module {
    module!(ID, NAME, on_message)
}

#[event]
async fn on_message(event: &MessageEvent) -> anyhow::Result<bool> {
    let content = event.message_content();
    if content.eq(NAME) {
        let mut result = vec!["菜单 (请直接回复功能名) : ".to_owned()];
        for m in all_modules().as_ref() {
            if m.name != "" {
                result.push(format!("\n ❤️ {}", m.name));
            }
        }
        event
            .send_message_to_source(result.join("").parse_message_chain())
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}
