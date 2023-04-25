use super::events::*;
use async_trait::async_trait;
use ricq::client::event::{
    ClientDisconnect, DeleteFriendEvent, FriendAudioMessageEvent, FriendMessageEvent,
    FriendMessageRecallEvent, FriendPokeEvent, GroupAudioMessageEvent, GroupDisbandEvent,
    GroupLeaveEvent, GroupMessageEvent, GroupMessageRecallEvent, GroupMuteEvent,
    GroupNameUpdateEvent, GroupPokeEvent, GroupTempMessageEvent, JoinGroupRequestEvent,
    MemberPermissionChangeEvent, NewFriendEvent, NewFriendRequestEvent, NewMemberEvent,
    SelfInvitedEvent,
};

pub struct ModuleEventHandler {
    pub name: String,
    pub process: ModuleEventProcess,
}

pub enum ModuleEventProcess {
    GroupMessage(Box<dyn GroupMessageEventProcess>),
    FriendMessage(Box<dyn FriendMessageEventProcess>),
    GroupTempMessage(Box<dyn GroupTempMessageEventProcess>),
    JoinGroupRequest(Box<dyn JoinGroupRequestEventProcess>),
    NewFriendRequest(Box<dyn NewFriendRequestEventProcess>),

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
    ConnectedAndOnline(Box<dyn ConnectedAndOnlineEventProcess>),
    DisconnectAndOffline(Box<dyn DisconnectedAndOfflineEventProcess>),

    GroupDisband(Box<dyn GroupDisbandEventProcess>),
    MemberPermissionChange(Box<dyn MemberPermissionChangeEventProcess>),
    NewMember(Box<dyn NewMemberEventProcess>),
    SelfInvited(Box<dyn SelfInvitedEventProcess>),

    GroupAudioMessage(Box<dyn GroupAudioMessageEventProcess>),
    FriendAudioMessage(Box<dyn FriendAudioMessageEventProcess>),
    ClientDisconnect(Box<dyn ClientDisconnectProcess>),
    GroupPoke(Box<dyn GroupPokeEventProcess>),
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
process_trait!(GroupTempMessageEventProcess, GroupTempMessageEvent);

process_trait!(JoinGroupRequestEventProcess, JoinGroupRequestEvent);
process_trait!(NewFriendRequestEventProcess, NewFriendRequestEvent);

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

process_trait!(ConnectedAndOnlineEventProcess, ConnectedAndOnlineEvent);
process_trait!(
    DisconnectedAndOfflineEventProcess,
    DisconnectedAndOfflineEvent
);

process_trait!(GroupDisbandEventProcess, GroupDisbandEvent);
process_trait!(
    MemberPermissionChangeEventProcess,
    MemberPermissionChangeEvent
);
process_trait!(NewMemberEventProcess, NewMemberEvent);
process_trait!(SelfInvitedEventProcess, SelfInvitedEvent);
process_trait!(GroupAudioMessageEventProcess, GroupAudioMessageEvent);
process_trait!(FriendAudioMessageEventProcess, FriendAudioMessageEvent);
process_trait!(ClientDisconnectProcess, ClientDisconnect);
process_trait!(GroupPokeEventProcess, GroupPokeEvent);
