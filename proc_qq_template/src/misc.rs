use proc_qq::re_export::rs_qq::client::event::GroupMuteEvent;
use proc_qq::{event, module, Module};

static ID: &'static str = "misc";
static NAME: &'static str = "";

pub(crate) fn module() -> Module {
    module!(ID, NAME, group_mute)
}

#[event]
async fn group_mute(event: &GroupMuteEvent) -> anyhow::Result<bool> {
    event.group_mute.target_uin;
    tracing::info!("禁言 : {}", event.group_mute.target_uin);
    Ok(false)
}
