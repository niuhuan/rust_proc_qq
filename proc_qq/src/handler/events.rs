use rq_engine::msg::MessageChain;
use rq_engine::{RQError, RQResult};
pub use rs_qq::client::event::{
    DeleteFriendEvent, FriendMessageEvent, FriendMessageRecallEvent, FriendPokeEvent,
    FriendRequestEvent, GroupLeaveEvent, GroupMessageEvent, GroupMessageRecallEvent,
    GroupMuteEvent, GroupNameUpdateEvent, GroupRequestEvent, KickedOfflineEvent, MSFOfflineEvent,
    NewFriendEvent, TempMessageEvent,
};
use std::sync::Arc;

pub struct LoginEvent {
    pub uin: i64,
}

pub enum MessageEvent {
    GroupMessage(GroupMessageEvent),
    FriendMessage(FriendMessageEvent),
    TempMessage(TempMessageEvent),
}

impl MessageEvent {
    pub fn client(&self) -> Arc<rs_qq::Client> {
        match self {
            MessageEvent::GroupMessage(e) => e.client.clone(),
            MessageEvent::FriendMessage(e) => e.client.clone(),
            MessageEvent::TempMessage(e) => e.client.clone(),
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
            MessageEvent::TempMessage(_) => true,
            _ => false,
        }
    }
    pub fn as_temp_message(&self) -> RQResult<&'_ TempMessageEvent> {
        match self {
            MessageEvent::TempMessage(temp_message) => RQResult::Ok(temp_message),
            _ => RQResult::Err(RQError::Other("Not is a group message".to_owned())),
        }
    }
    pub fn from_uin(&self) -> i64 {
        match self {
            MessageEvent::GroupMessage(message) => message.message.from_uin,
            MessageEvent::FriendMessage(message) => message.message.from_uin,
            MessageEvent::TempMessage(message) => message.message.from_uin,
        }
    }
    pub fn elements(&self) -> MessageChain {
        match self {
            MessageEvent::GroupMessage(message) => &message.message.elements,
            MessageEvent::FriendMessage(message) => &message.message.elements,
            MessageEvent::TempMessage(message) => &message.message.elements,
        }
        .clone()
    }
}

pub struct ConnectedAndOnlineEvent {}

pub struct DisconnectedAndOfflineEvent {}
