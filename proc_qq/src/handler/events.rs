use rq_engine::{RQError, RQResult};
pub use rs_qq::client::event::{
    DeleteFriendEvent, FriendMessageRecallEvent, FriendPokeEvent, FriendRequestEvent,
    GroupLeaveEvent, GroupMessageEvent, GroupMessageRecallEvent, GroupMuteEvent,
    GroupNameUpdateEvent, GroupRequestEvent, NewFriendEvent, PrivateMessageEvent, TempMessageEvent,
};
use std::sync::Arc;

pub struct LoginEvent {
    pub uin: i64,
}

pub enum MessageEvent<'a> {
    GroupMessage(&'a GroupMessageEvent),
    PrivateMessage(&'a PrivateMessageEvent),
    TempMessage(&'a TempMessageEvent),
}

impl MessageEvent<'_> {
    pub fn client(&self) -> Arc<rs_qq::Client> {
        match self {
            MessageEvent::GroupMessage(e) => e.client.clone(),
            MessageEvent::PrivateMessage(e) => e.client.clone(),
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
            MessageEvent::PrivateMessage(_) => true,
            _ => false,
        }
    }
    pub fn as_private_message(&self) -> RQResult<&'_ PrivateMessageEvent> {
        match self {
            MessageEvent::PrivateMessage(private_message) => RQResult::Ok(private_message),
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
            MessageEvent::PrivateMessage(message) => message.message.from_uin,
            MessageEvent::TempMessage(message) => message.message.from_uin,
        }
    }
}
