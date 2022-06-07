use super::events::*;
use async_trait::async_trait;
use ricq::client::event::{
    DeleteFriendEvent, FriendMessageEvent, FriendMessageRecallEvent, FriendPokeEvent,
    FriendRequestEvent, GroupDisbandEvent, GroupLeaveEvent, GroupMessageEvent,
    GroupMessageRecallEvent, GroupMuteEvent, GroupNameUpdateEvent, GroupRequestEvent,
    GroupTempMessageEvent, MemberPermissionChangeEvent, NewFriendEvent, NewMemberEvent,
    SelfInvitedEvent,
};

pub struct ModuleInfo {
    pub module_id: String,
    pub module_name: String,
    pub handle_name: String,
}

pub enum EventResult {
    Process(ModuleInfo),
    Exception(ModuleInfo, anyhow::Error),
}

pub struct EventResultHandler {
    pub name: String,
    pub process: ResultProcess,
}

macro_rules! error_trait {
    ($name:ident, $event:path) => {
        #[async_trait]
        pub trait $name: Sync + Send {
            async fn handle(&self, event: &$event, result: &EventResult) -> anyhow::Result<bool>;
        }
    };
}

pub enum ResultProcess {
    GroupMessage(Box<dyn GroupMessageResultHandler>),
    FriendMessage(Box<dyn FriendMessageResultHandler>),
    GroupTempMessage(Box<dyn GroupTempMessageResultHandler>),
    GroupRequest(Box<dyn GroupRequestResultHandler>),
    FriendRequest(Box<dyn FriendRequestResultHandler>),

    NewFriend(Box<dyn NewFriendResultHandler>),
    FriendPoke(Box<dyn FriendPokeResultHandler>),
    DeleteFriend(Box<dyn DeleteFriendResultHandler>),

    GroupMute(Box<dyn GroupMuteResultHandler>),
    GroupLeave(Box<dyn GroupLeaveResultHandler>),
    GroupNameUpdate(Box<dyn GroupNameUpdateResultHandler>),

    GroupMessageRecall(Box<dyn GroupMessageRecallResultHandler>),
    FriendMessageRecall(Box<dyn FriendMessageRecallResultHandler>),

    MSFOffline(Box<dyn MSFOfflineResultHandler>),
    KickedOffline(Box<dyn KickedOfflineResultHandler>),

    LoginEvent(Box<dyn LoginResultHandler>),
    Message(Box<dyn MessageResultHandler>),
    ConnectedAndOnline(Box<dyn ConnectedAndOnlineResultHandler>),
    DisconnectAndOffline(Box<dyn DisconnectedAndOfflineResultHandler>),

    GroupDisband(Box<dyn GroupDisbandResultHandler>),
    MemberPermissionChange(Box<dyn MemberPermissionChangeResultHandler>),
    NewMember(Box<dyn NewMemberResultHandler>),
    SelfInvited(Box<dyn SelfInvitedResultHandler>),

    OnlyResult(Box<dyn OnlyResultHandler>),
}

error_trait!(GroupMessageResultHandler, GroupMessageEvent);
error_trait!(FriendMessageResultHandler, FriendMessageEvent);
error_trait!(GroupTempMessageResultHandler, GroupTempMessageEvent);

error_trait!(GroupRequestResultHandler, GroupRequestEvent);
error_trait!(FriendRequestResultHandler, FriendRequestEvent);

error_trait!(NewFriendResultHandler, NewFriendEvent);
error_trait!(FriendPokeResultHandler, FriendPokeEvent);
error_trait!(DeleteFriendResultHandler, DeleteFriendEvent);

error_trait!(GroupMuteResultHandler, GroupMuteEvent);
error_trait!(GroupLeaveResultHandler, GroupLeaveEvent);
error_trait!(GroupNameUpdateResultHandler, GroupNameUpdateEvent);

error_trait!(GroupMessageRecallResultHandler, GroupMessageRecallEvent);
error_trait!(FriendMessageRecallResultHandler, FriendMessageRecallEvent);

error_trait!(MSFOfflineResultHandler, MSFOfflineEvent);
error_trait!(KickedOfflineResultHandler, KickedOfflineEvent);

error_trait!(LoginResultHandler, LoginEvent);
error_trait!(MessageResultHandler, MessageEvent);

error_trait!(ConnectedAndOnlineResultHandler, ConnectedAndOnlineEvent);
error_trait!(
    DisconnectedAndOfflineResultHandler,
    DisconnectedAndOfflineEvent
);

error_trait!(GroupDisbandResultHandler, GroupDisbandEvent);
error_trait!(
    MemberPermissionChangeResultHandler,
    MemberPermissionChangeEvent
);
error_trait!(NewMemberResultHandler, NewMemberEvent);
error_trait!(SelfInvitedResultHandler, SelfInvitedEvent);

#[async_trait]
pub trait OnlyResultHandler: Sync + Send {
    async fn handle(&self, result: &EventResult) -> anyhow::Result<bool>;
}
