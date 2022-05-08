use proc_qq::re_exports::async_trait::async_trait;
use proc_qq::re_exports::ricq::msg::MessageChain;
use proc_qq::re_exports::ricq::RQResult;
use proc_qq::re_exports::ricq_core::msg::elem::At;
use proc_qq::re_exports::ricq_core::structs::GroupMemberInfo;
use proc_qq::{
    ClientTrait, GroupMessageEvent, GroupTrait, MessageChainParseTrait, MessageChainTrait,
    MessageEvent, MessageSendToSourceTrait, TextEleParseTrait,
};
pub(crate) mod ffmpeg_cmd;
pub(crate) mod local;

#[async_trait]
pub(crate) trait CanReply {
    async fn make_reply_chain(&self) -> MessageChain;
    async fn reply_text(&self, text: &str) -> RQResult<()>;
    async fn reply_raw_text(&self, text: &str) -> RQResult<()>;
}

#[async_trait]
impl CanReply for GroupMessageEvent {
    async fn make_reply_chain(&self) -> MessageChain {
        let group = self
            .client
            .must_find_group(self.message.group_code, true)
            .await;
        if group.is_ok() {
            let group = group.unwrap();
            let member = group.must_find_member(self.message.from_uin).await;
            if member.is_ok() {
                let member = member.unwrap();
                return MessageChain::default()
                    .append(at_member(&member))
                    .append("\n\n".parse_text());
            }
        }
        MessageChain::default().append(At::new(self.message.from_uin))
    }

    async fn reply_text(&self, text: &str) -> RQResult<()> {
        self.send_message_to_source(self.make_reply_chain().await.append(text.parse_text()))
            .await?;
        RQResult::Ok(())
    }

    async fn reply_raw_text(&self, text: &str) -> RQResult<()> {
        self.send_message_to_source(text.parse_message_chain())
            .await?;
        RQResult::Ok(())
    }
}

#[async_trait]
impl CanReply for MessageEvent {
    async fn make_reply_chain(&self) -> MessageChain {
        match self {
            MessageEvent::GroupMessage(group_message) => group_message.make_reply_chain().await,
            _ => MessageChain::default(),
        }
    }
    async fn reply_text(&self, text: &str) -> RQResult<()> {
        self.send_message_to_source(self.make_reply_chain().await.append(text.parse_text()))
            .await?;
        RQResult::Ok(())
    }

    async fn reply_raw_text(&self, text: &str) -> RQResult<()> {
        self.send_message_to_source(text.parse_message_chain())
            .await?;
        RQResult::Ok(())
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
