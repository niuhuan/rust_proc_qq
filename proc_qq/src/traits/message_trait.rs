use async_trait::async_trait;
use rq_engine::msg::elem::Text;
use rq_engine::msg::MessageChain;
use rq_engine::structs::{GroupMessage, MessageReceipt, PrivateMessage, TempMessage};
use rq_engine::RQResult;
use rs_qq::client::event::{GroupMessageEvent, PrivateMessageEvent, TempMessageEvent};

use crate::{ClientTrait, MessageEvent};

pub enum MessageSource {
    // Group(group_code,uin)
    Group(i64, i64),
    // Private(uin)
    Private(i64),
    // Temp(group_code,uin)
    Temp(Option<i64>, i64),
}

pub trait MessageSourceTrait: Send + Sync {
    fn message_source(&self) -> MessageSource;
}

pub trait MessageContentTrait: Send + Sync {
    fn message_content(&self) -> String;
}

#[async_trait]
pub trait MessageSendToSourceTrait: Send + Sync {
    async fn send_message_to_source<S: Into<MessageChain> + Send + Sync>(
        &self,
        message: S,
    ) -> RQResult<MessageReceipt>;
}

pub trait TextEleParseTrait {
    fn parse_text(self) -> Text;
}

pub trait MessageChainParseTrait {
    fn parse_message_chain(self) -> MessageChain;
}

impl MessageContentTrait for MessageChain {
    fn message_content(&self) -> String {
        self.to_string()
    }
}

impl MessageSourceTrait for GroupMessage {
    fn message_source(&self) -> MessageSource {
        MessageSource::Group(self.group_code, self.from_uin)
    }
}

impl MessageContentTrait for GroupMessage {
    fn message_content(&self) -> String {
        self.elements.message_content()
    }
}

impl MessageSourceTrait for GroupMessageEvent {
    fn message_source(&self) -> MessageSource {
        self.message.message_source()
    }
}

impl MessageContentTrait for GroupMessageEvent {
    fn message_content(&self) -> String {
        self.message.message_content()
    }
}

#[async_trait]
impl MessageSendToSourceTrait for GroupMessageEvent {
    async fn send_message_to_source<S: Into<MessageChain> + Send + Sync>(
        &self,
        message: S,
    ) -> RQResult<MessageReceipt> {
        self.client.send_message_to_source(self, message).await
    }
}

impl MessageSourceTrait for PrivateMessage {
    fn message_source(&self) -> MessageSource {
        MessageSource::Private(self.from_uin)
    }
}

impl MessageContentTrait for PrivateMessage {
    fn message_content(&self) -> String {
        self.elements.to_string()
    }
}

impl MessageSourceTrait for PrivateMessageEvent {
    fn message_source(&self) -> MessageSource {
        self.message.message_source()
    }
}

impl MessageContentTrait for PrivateMessageEvent {
    fn message_content(&self) -> String {
        self.message.message_content()
    }
}

#[async_trait]
impl MessageSendToSourceTrait for PrivateMessageEvent {
    async fn send_message_to_source<S: Into<MessageChain> + Send + Sync>(
        &self,
        message: S,
    ) -> RQResult<MessageReceipt> {
        self.client.send_message_to_source(self, message).await
    }
}

impl MessageSourceTrait for TempMessage {
    fn message_source(&self) -> MessageSource {
        MessageSource::Temp(self.group_code, self.from_uin)
    }
}

impl MessageContentTrait for TempMessage {
    fn message_content(&self) -> String {
        self.elements.to_string()
    }
}

impl MessageSourceTrait for TempMessageEvent {
    fn message_source(&self) -> MessageSource {
        self.message.message_source()
    }
}

impl MessageContentTrait for TempMessageEvent {
    fn message_content(&self) -> String {
        self.message.message_content()
    }
}

#[async_trait]
impl MessageSendToSourceTrait for TempMessageEvent {
    async fn send_message_to_source<S: Into<MessageChain> + Send + Sync>(
        &self,
        message: S,
    ) -> RQResult<MessageReceipt> {
        self.client.send_message_to_source(self, message).await
    }
}

impl MessageSourceTrait for MessageEvent<'_> {
    fn message_source(&self) -> MessageSource {
        match self {
            MessageEvent::GroupMessage(event) => event.message_source(),
            MessageEvent::PrivateMessage(event) => event.message_source(),
            MessageEvent::TempMessage(event) => event.message_source(),
        }
    }
}

impl MessageContentTrait for MessageEvent<'_> {
    fn message_content(&self) -> String {
        match self {
            MessageEvent::GroupMessage(event) => event.message_content(),
            MessageEvent::PrivateMessage(event) => event.message_content(),
            MessageEvent::TempMessage(event) => event.message_content(),
        }
    }
}

#[async_trait]
impl MessageSendToSourceTrait for MessageEvent<'_> {
    async fn send_message_to_source<S: Into<MessageChain> + Send + Sync>(
        &self,
        message: S,
    ) -> RQResult<MessageReceipt> {
        match self {
            MessageEvent::GroupMessage(event) => event.send_message_to_source(message).await,
            MessageEvent::PrivateMessage(event) => event.send_message_to_source(message).await,
            MessageEvent::TempMessage(event) => event.send_message_to_source(message).await,
        }
    }
}

impl TextEleParseTrait for String {
    fn parse_text(self) -> Text {
        Text::new(self)
    }
}

impl TextEleParseTrait for &str {
    fn parse_text(self) -> Text {
        Text::new(self.to_owned())
    }
}

impl MessageChainParseTrait for String {
    fn parse_message_chain(self) -> MessageChain {
        MessageChain::new(self.parse_text())
    }
}

impl MessageChainParseTrait for &str {
    fn parse_message_chain(self) -> MessageChain {
        MessageChain::new(self.parse_text())
    }
}
