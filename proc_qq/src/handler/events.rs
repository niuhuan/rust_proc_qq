pub use ricq::client::event::{
    DeleteFriendEvent, FriendMessageEvent, FriendMessageRecallEvent, FriendPokeEvent,
    FriendRequestEvent, GroupLeaveEvent, GroupMessageEvent, GroupMessageRecallEvent,
    GroupMuteEvent, GroupNameUpdateEvent, GroupRequestEvent, GroupTempMessageEvent,
    KickedOfflineEvent, MSFOfflineEvent, NewFriendEvent,
};
use ricq_core::msg::MessageChain;
use ricq_core::{RQError, RQResult};
use std::sync::Arc;

pub struct LoginEvent {
    pub uin: i64,
}

pub enum MessageEvent {
    GroupMessage(GroupMessageEvent),
    FriendMessage(FriendMessageEvent),
    GroupTempMessage(GroupTempMessageEvent),
}

impl MessageEvent {
    pub fn client(&self) -> Arc<ricq::Client> {
        match self {
            MessageEvent::GroupMessage(e) => e.client.clone(),
            MessageEvent::FriendMessage(e) => e.client.clone(),
            MessageEvent::GroupTempMessage(e) => e.client.clone(),
        }
    }
    pub fn is_group_message(&self) -> bool {
        match self {
            MessageEvent::GroupMessage(_) => true,
            _ => false,
        }
    }
    pub fn as_group_message(&self) -> RQResult<&'_ GroupMessageEvent> {
        match self {
            MessageEvent::GroupMessage(group_message) => RQResult::Ok(group_message),
            _ => RQResult::Err(RQError::Other("Not is a group message".to_owned())),
        }
    }
    pub fn is_private_message(&self) -> bool {
        match self {
            MessageEvent::FriendMessage(_) => true,
            _ => false,
        }
    }
    pub fn as_private_message(&self) -> RQResult<&'_ FriendMessageEvent> {
        match self {
            MessageEvent::FriendMessage(private_message) => RQResult::Ok(private_message),
            _ => RQResult::Err(RQError::Other("Not is a group message".to_owned())),
        }
    }
    pub fn is_temp_message(&self) -> bool {
        match self {
            MessageEvent::GroupTempMessage(_) => true,
            _ => false,
        }
    }
    pub fn as_temp_message(&self) -> RQResult<&'_ GroupTempMessageEvent> {
        match self {
            MessageEvent::GroupTempMessage(temp_message) => RQResult::Ok(temp_message),
            _ => RQResult::Err(RQError::Other("Not is a group message".to_owned())),
        }
    }
    pub fn from_uin(&self) -> i64 {
        match self {
            MessageEvent::GroupMessage(message) => message.message.from_uin,
            MessageEvent::FriendMessage(message) => message.message.from_uin,
            MessageEvent::GroupTempMessage(message) => message.message.from_uin,
        }
    }
    pub fn elements(&self) -> MessageChain {
        match self {
            MessageEvent::GroupMessage(message) => &message.message.elements,
            MessageEvent::FriendMessage(message) => &message.message.elements,
            MessageEvent::GroupTempMessage(message) => &message.message.elements,
        }
        .clone()
    }
}

pub struct ConnectedAndOnlineEvent {}

pub struct DisconnectedAndOfflineEvent {}
