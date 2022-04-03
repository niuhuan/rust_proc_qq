use super::events::*;
use async_trait::async_trait;
use rs_qq::client::event::{
    DeleteFriendEvent, FriendMessageEvent, FriendMessageRecallEvent, FriendPokeEvent,
    FriendRequestEvent, GroupLeaveEvent, GroupMessageEvent, GroupMessageRecallEvent,
    GroupMuteEvent, GroupNameUpdateEvent, GroupRequestEvent, NewFriendEvent, TempMessageEvent,
};

#[macro_export]
macro_rules! module {
    ($id:expr,$name:expr $(, $x:tt)* $(,)?) => (
        ::proc_qq::Module {
            id: $id.to_owned(),
            name: $name.to_owned(),
            handles: vec![$($x {}.into(),)*],
        }
    );
}

pub struct ModuleEventHandler {
    pub name: String,
    pub process: ModuleEventProcess,
}

pub enum ModuleEventProcess {
    GroupMessage(Box<dyn GroupMessageEventProcess>),
    FriendMessage(Box<dyn FriendMessageEventProcess>),
    TempMessage(Box<dyn TempMessageEventProcess>),
    GroupRequest(Box<dyn GroupRequestEventProcess>),
    FriendRequest(Box<dyn FriendRequestEventProcess>),

    NewFriend(Box<dyn NewFriendEventProcess>),
    FriendPoke(Box<dyn FriendPokeEventProcess>),
    DeleteFriend(Box<dyn DeleteFriendEventProcess>),

    GroupMute(Box<dyn GroupMuteEventProcess>),
    GroupLeave(Box<dyn GroupLeaveEventProcess>),
    GroupNameUpdate(Box<dyn GroupNameUpdateEventProcess>),

    GroupMessageRecall(Box<dyn GroupMessageRecallEventProcess>),
    FriendMessageRecall(Box<dyn FriendMessageRecallEventProcess>),

    MSFOffline(Box<dyn MSFOfflineEventProcess>),
    KickedOffline(Box<dyn KickedOfflineEventProcess>),

    LoginEvent(Box<dyn LoginEventProcess>),
    Message(Box<dyn MessageEventProcess>),
}

macro_rules! process_trait {
    ($name:ident, $event:path) => {
        #[async_trait]
        pub trait $name: Sync + Send {
            async fn handle(&self, event: &$event) -> anyhow::Result<bool>;
        }
    };
}

process_trait!(GroupMessageEventProcess, GroupMessageEvent);
process_trait!(FriendMessageEventProcess, FriendMessageEvent);
process_trait!(TempMessageEventProcess, TempMessageEvent);

process_trait!(GroupRequestEventProcess, GroupRequestEvent);
process_trait!(FriendRequestEventProcess, FriendRequestEvent);

process_trait!(NewFriendEventProcess, NewFriendEvent);
process_trait!(FriendPokeEventProcess, FriendPokeEvent);
process_trait!(DeleteFriendEventProcess, DeleteFriendEvent);

process_trait!(GroupMuteEventProcess, GroupMuteEvent);
process_trait!(GroupLeaveEventProcess, GroupLeaveEvent);
process_trait!(GroupNameUpdateEventProcess, GroupNameUpdateEvent);

process_trait!(GroupMessageRecallEventProcess, GroupMessageRecallEvent);
process_trait!(FriendMessageRecallEventProcess, FriendMessageRecallEvent);

process_trait!(MSFOfflineEventProcess, MSFOfflineEvent);
process_trait!(KickedOfflineEventProcess, KickedOfflineEvent);

process_trait!(LoginEventProcess, LoginEvent);
process_trait!(MessageEventProcess, MessageEvent);
