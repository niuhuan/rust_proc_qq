use proc_qq::re_exports::async_trait::async_trait;
use proc_qq::re_exports::ricq::msg::MessageChain;
use proc_qq::re_exports::ricq::RQResult;
use proc_qq::re_exports::ricq_core::msg::elem::At;
use proc_qq::{
    GroupMessageEvent, MessageChainAppendTrait, MessageChainParseTrait, MessageEvent,
    MessageSendToSourceTrait, TextEleParseTrait,
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
        let mut at = At::new(self.inner.from_uin);
        at.display = format!("@{}", self.inner.group_card);
        MessageChain::default()
            .append(at)
            .append("\n\n".parse_text())
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
