use proc_qq::re_exports::async_trait::async_trait;
use proc_qq::re_exports::rq_engine::msg::elem::At;
use proc_qq::re_exports::rq_engine::structs::GroupMemberInfo;
use proc_qq::re_exports::rs_qq::msg::MessageChain;
use proc_qq::{ClientTrait, GroupMessageEvent, GroupTrait, MessageChainTrait, MessageEvent};

#[async_trait]
pub(crate) trait ReplyChain {
    async fn reply_chain(&self) -> MessageChain;
}

#[async_trait]
impl ReplyChain for GroupMessageEvent {
    async fn reply_chain(&self) -> MessageChain {
        let group = self
            .client
            .must_find_group(self.message.group_code, true)
            .await;
        if group.is_ok() {
            let group = group.unwrap();
            let member = group.must_find_member(self.message.from_uin).await;
            if member.is_ok() {
                let member = member.unwrap();
                return MessageChain::default().append(at_member(&member));
            }
        }
        MessageChain::default().append(At::new(self.message.from_uin))
    }
}

#[async_trait]
impl ReplyChain for MessageEvent<'_> {
    async fn reply_chain(&self) -> MessageChain {
        match self {
            MessageEvent::GroupMessage(group_message) => group_message.reply_chain().await,
            _ => MessageChain::default(),
        }
    }
}

pub(crate) fn at_member(member: &GroupMemberInfo) -> At {
    let mut at = At::new(member.uin);
    at.display = format!(
        "@{}",
        if member.card_name != "" {
            &member.card_name
        } else {
            &member.nickname
        }
    );
    at
}
