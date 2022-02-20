use proc_qq::re_export::rs_qq::client::event::{GroupMessageEvent, PrivateMessageEvent};
use proc_qq::re_export::rs_qq::msg::elem::Text;
use proc_qq::re_export::rs_qq::msg::MessageChain;
use proc_qq::{event, module, Module};

static ID: &'static str = "menu";
static NAME: &'static str = "菜单";
static MENU: &'static str = "菜单 (请直接回复功能名) : \n ❤️ 图库";

pub(crate) fn module() -> Module {
    module!(ID, NAME, group_message, private_message)
}

#[event]
async fn group_message(event: &GroupMessageEvent) -> anyhow::Result<bool> {
    let content = event.message.elements.to_string();
    if content.eq(NAME) {
        let chain = MessageChain::new(Text::new(MENU.to_owned()));
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
async fn private_message(event: &PrivateMessageEvent) -> anyhow::Result<bool> {
    let content = event.message.elements.to_string();
    if content.eq(NAME) {
        let chain = MessageChain::new(Text::new(MENU.to_owned()));
        event
            .client
            .send_private_message(event.message.from_uin, chain)
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}
