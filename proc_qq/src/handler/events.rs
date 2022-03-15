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
    pub fn is_private_message(&self) -> bool {
        match self {
            MessageEvent::PrivateMessage(_) => true,
            _ => false,
        }
    }
    pub fn is_temp_message(&self) -> bool {
        match self {
            MessageEvent::TempMessage(_) => true,
            _ => false,
        }
    }
}
