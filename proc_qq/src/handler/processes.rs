use super::events::*;
use async_trait::async_trait;
use rs_qq::client::event::{
    DeleteFriendEvent, FriendMessageRecallEvent, FriendPokeEvent, FriendRequestEvent,
    GroupLeaveEvent, GroupMessageEvent, GroupMessageRecallEvent, GroupMuteEvent,
    GroupNameUpdateEvent, GroupRequestEvent, NewFriendEvent, PrivateMessageEvent, TempMessageEvent,
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
    LoginEvent(Box<dyn LoginEventProcess>),

    GroupMessage(Box<dyn GroupMessageEventProcess>),
    PrivateMessage(Box<dyn PrivateMessageEventProcess>),
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

process_trait!(LoginEventProcess, LoginEvent);

process_trait!(GroupMessageEventProcess, GroupMessageEvent);
process_trait!(PrivateMessageEventProcess, PrivateMessageEvent);
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

process_trait!(MessageEventProcess, MessageEvent);
