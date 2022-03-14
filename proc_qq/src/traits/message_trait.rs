use async_trait::async_trait;
use rq_engine::msg::elem::{FlashImage, FriendImage, GroupImage, Text};
use rq_engine::msg::MessageChain;
use rq_engine::pb::msg::elem::Elem;
use rq_engine::structs::{GroupMessage, MessageReceipt, PrivateMessage, TempMessage};
use rq_engine::{RQError, RQResult};
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

pub enum UploadImage {
    FriendImage(FriendImage),
    GroupImage(GroupImage),
}

impl Into<Vec<Elem>> for UploadImage {
    fn into(self) -> Vec<Elem> {
        match self {
            UploadImage::FriendImage(i) => i.into(),
            UploadImage::GroupImage(i) => i.into(),
        }
    }
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

    async fn upload_image_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        data: S,
    ) -> RQResult<UploadImage>;
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

    async fn upload_image_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        data: S,
    ) -> RQResult<UploadImage> {
        Ok(UploadImage::GroupImage(
            self.client
                .upload_group_image(self.message.group_code, data.into())
                .await?,
        ))
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

    async fn upload_image_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        data: S,
    ) -> RQResult<UploadImage> {
        Ok(UploadImage::FriendImage(
            self.client
                .upload_private_image(self.message.from_uin, data.into())
                .await?,
        ))
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

    async fn upload_image_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        _: S,
    ) -> RQResult<UploadImage> {
        RQResult::Err(RQError::Other(
            "tmp message not supported upload image".to_owned(),
        ))
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

    async fn upload_image_to_source<S: Into<Vec<u8>> + Send + Sync>(
        &self,
        data: S,
    ) -> RQResult<UploadImage> {
        match self {
            MessageEvent::GroupMessage(event) => event.upload_image_to_source(data).await,
            MessageEvent::PrivateMessage(event) => event.upload_image_to_source(data).await,
            MessageEvent::TempMessage(event) => event.upload_image_to_source(data).await,
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

impl MessageChainParseTrait for FriendImage {
    fn parse_message_chain(self) -> MessageChain {
        let mut chain = MessageChain::default();
        chain.push(self);
        chain
    }
}

impl MessageChainParseTrait for GroupImage {
    fn parse_message_chain(self) -> MessageChain {
        let mut chain = MessageChain::default();
        chain.push(self);
        chain
    }
}

impl MessageChainParseTrait for FlashImage {
    fn parse_message_chain(self) -> MessageChain {
        let mut chain = MessageChain::default();
        chain.push(self);
        chain
    }
}

impl MessageChainParseTrait for UploadImage {
    fn parse_message_chain(self) -> MessageChain {
        match self {
            UploadImage::FriendImage(i) => i.parse_message_chain(),
            UploadImage::GroupImage(i) => i.parse_message_chain(),
        }
    }
}
