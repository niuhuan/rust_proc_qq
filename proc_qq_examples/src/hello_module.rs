use proc_qq::re_export::rs_qq::client::event::{GroupMessageEvent, PrivateMessageEvent};
use proc_qq::re_export::rs_qq::msg::elem::Text;
use proc_qq::re_export::rs_qq::msg::MessageChain;
use proc_qq::{event, module, Module};

pub(crate) fn module() -> Module {
    module!("hello", "你好", group_hello, private_hello)
}

#[event]
async fn group_hello(event: &GroupMessageEvent) -> anyhow::Result<bool> {
    let content = event.message.elements.to_string();
    if content.eq("你好") {
        let chain = MessageChain::new(Text::new("世界".to_string()));
        event
            .client
            .send_group_message(event.message.group_code, chain)
            .await?;
        Ok(true)
    } else if content.eq("RC") {
        let chain = MessageChain::new(Text::new("NB".to_string()));
        event
            .client
            .send_group_message(event.message.group_code, chain)
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[event]
async fn private_hello(event: &PrivateMessageEvent) -> anyhow::Result<bool> {
    let content = event.message.elements.to_string();
    if content.eq("你好") {
        let chain = MessageChain::new(Text::new("世界".to_string()));
        event
            .client
            .send_private_message(event.message.from_uin, chain)
            .await?;
        Ok(true)
    } else if content.eq("RC") {
        let chain = MessageChain::new(Text::new("NB".to_string()));
        event
            .client
            .send_private_message(event.message.from_uin, chain)
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}
