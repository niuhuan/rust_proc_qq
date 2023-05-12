use async_trait::async_trait;
#[cfg(feature = "event_args")]
pub use event_args::*;
pub use events::*;
pub use processes::*;
pub use results::*;
use ricq::handler::{Handler, QEvent};
use std::sync::Arc;

#[cfg(feature = "event_args")]
mod event_args;
mod events;
mod processes;
mod results;

pub(crate) struct ClientHandler {
    pub(crate) modules: Arc<Vec<Module>>,
    pub(crate) result_handlers: Arc<Vec<EventResultHandler>>,
}

impl ClientHandler {}

enum MapResult<'a> {
    None,
    Process(&'a str, &'a str),
    Exception(&'a str, &'a str),
}

macro_rules! map_result {
    ($self:expr, $event:expr, $result_handler:path, $event_result:expr) => {
        for h in $self.result_handlers.as_ref() {
            let mut hand = false;
            match &h.process {
                $result_handler(e) => match e.handle($event, $event_result).await {
                    Ok(b) => {
                        hand = b;
                    }
                    Err(err) => {
                        tracing::error!(" 出现错误 : {:?}", err);
                        hand = true;
                    }
                },
                ResultProcess::OnlyResult(e) => match e.handle($event_result).await {
                    Ok(b) => {
                        hand = b;
                    }
                    Err(err) => {
                        tracing::error!(" 出现错误 : {:?}", err);
                        hand = true;
                    }
                },
                _ => (),
            }
            if hand {
                break;
            }
        }
    };
}

macro_rules! map_handlers {
    ($self:expr $(,$event:expr, $process:path, $result_handler:path)* $(,)?) => {{
        let mut result = MapResult::None;
        for m in $self.modules.as_ref() {
            for h in &m.handles {
                match &h.process {
                    $(
                    $process(e) => match e.handle($event).await {
                        Ok(b) => {
                            if b {
                                result = MapResult::Process(&m.id, &h.name);
                                let event_result = EventResult::Process(
                                    ModuleInfo{
                                        module_id: m.id.clone(),
                                        module_name: m.name.clone(),
                                        handle_name: h.name.clone(),
                                    },
                                );
                                map_result!($self, $event, $result_handler, &event_result);
                            }
                        }
                        Err(err) => {
                            tracing::error!(" 出现错误 : {:?}", err);
                            result = MapResult::Exception(&m.id, &h.name);
                            let event_result = EventResult::Exception(
                                ModuleInfo{
                                    module_id: m.id.clone(),
                                    module_name: m.name.clone(),
                                    handle_name: h.name.clone(),
                                },
                                err,
                            );
                            map_result!($self, $event, $result_handler, &event_result);
                        }
                    },
                    )*
                    _ => (),
                }
                if let MapResult::None = result {
                } else {
                    break;
                }
            }
            if let MapResult::None = result {
            } else {
                break;
            }
        }
        result
    }};
}

#[async_trait]
impl Handler for ClientHandler {
    async fn handle(&self, e: QEvent) {
        match e {
            QEvent::Login(event) => {
                tracing::debug!("LOGIN : (UIN={})", event,);
                let _ = map_handlers!(
                    &self,
                    &LoginEvent { uin: event },
                    ModuleEventProcess::LoginEvent,
                    ResultProcess::LoginEvent,
                );
            }
            QEvent::GroupMessage(event) => {
                tracing::debug!(
                    "(GROUP={}, UIN={}) MESSAGE : {}",
                    event.inner.group_code,
                    event.inner.from_uin,
                    event.inner.elements.to_string()
                );
                let me = MessageEvent::GroupMessage(event.clone());
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::GroupMessage,
                    ResultProcess::GroupMessage,
                    &me,
                    ModuleEventProcess::Message,
                    ResultProcess::Message,
                );
            }
            QEvent::FriendMessage(event) => {
                tracing::debug!(
                    "(UIN={}) MESSAGE : {}",
                    event.inner.from_uin,
                    event.inner.elements.to_string()
                );
                let me = MessageEvent::FriendMessage(event.clone());
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::FriendMessage,
                    ResultProcess::FriendMessage,
                    &me,
                    ModuleEventProcess::Message,
                    ResultProcess::Message,
                );
            }
            QEvent::GroupTempMessage(event) => {
                tracing::debug!(
                    "(UIN={}) MESSAGE : {}",
                    event.inner.from_uin,
                    event.inner.elements.to_string()
                );
                let me = MessageEvent::GroupTempMessage(event.clone());
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::GroupTempMessage,
                    ResultProcess::GroupTempMessage,
                    &me,
                    ModuleEventProcess::Message,
                    ResultProcess::Message,
                );
            }
            QEvent::GroupRequest(event) => {
                tracing::debug!(
                    "REQUEST (GROUP={}, UIN={}): {}",
                    event.inner.group_code,
                    event.inner.req_uin,
                    event.inner.message,
                );
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::JoinGroupRequest,
                    ResultProcess::JoinGroupRequest,
                );
            }
            QEvent::NewFriendRequest(event) => {
                tracing::debug!(
                    "REQUEST (UIN={}): {}",
                    event.inner.req_uin,
                    event.inner.message
                );
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::NewFriendRequest,
                    ResultProcess::NewFriendRequest,
                );
            }
            QEvent::NewFriend(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::NewFriend,
                    ResultProcess::NewFriend
                );
            }
            QEvent::FriendPoke(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::FriendPoke,
                    ResultProcess::FriendPoke
                );
            }
            QEvent::DeleteFriend(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::DeleteFriend,
                    ResultProcess::DeleteFriend
                );
            }
            QEvent::GroupMute(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::GroupMute,
                    ResultProcess::GroupMute
                );
            }
            QEvent::GroupLeave(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::GroupLeave,
                    ResultProcess::GroupLeave
                );
            }
            QEvent::GroupNameUpdate(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::GroupNameUpdate,
                    ResultProcess::GroupNameUpdate
                );
            }
            QEvent::GroupMessageRecall(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::GroupMessageRecall,
                    ResultProcess::GroupMessageRecall
                );
            }
            QEvent::FriendMessageRecall(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::FriendMessageRecall,
                    ResultProcess::FriendMessageRecall
                );
            }
            QEvent::MSFOffline(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::MSFOffline,
                    ResultProcess::MSFOffline
                );
            }
            QEvent::KickedOffline(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::KickedOffline,
                    ResultProcess::KickedOffline
                );
            }
            QEvent::GroupDisband(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::GroupDisband,
                    ResultProcess::GroupDisband
                );
            }
            QEvent::MemberPermissionChange(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::MemberPermissionChange,
                    ResultProcess::MemberPermissionChange
                );
            }
            QEvent::SelfInvited(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::SelfInvited,
                    ResultProcess::SelfInvited
                );
            }
            QEvent::GroupAudioMessage(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::GroupAudioMessage,
                    ResultProcess::GroupAudioMessage
                );
            }
            QEvent::FriendAudioMessage(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::FriendAudioMessage,
                    ResultProcess::FriendAudioMessage
                );
            }
            QEvent::NewMember(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::NewMember,
                    ResultProcess::NewMember
                );
            }
            QEvent::ClientDisconnect(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::ClientDisconnect,
                    ResultProcess::ClientDisconnect
                );
            }
            QEvent::GroupPoke(event) => {
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::GroupPoke,
                    ResultProcess::GroupPoke
                );
            }
        }
    }
}

pub struct Module {
    pub id: String,
    pub name: String,
    pub handles: Vec<ModuleEventHandler>,
}

pub(crate) struct EventSender {
    pub(crate) modules: Arc<Vec<Module>>,
    pub(crate) result_handlers: Arc<Vec<EventResultHandler>>,
}

impl EventSender {
    pub async fn send_connected_and_online(&self) -> anyhow::Result<()> {
        match map_handlers!(
            &self,
            &ConnectedAndOnlineEvent {},
            ModuleEventProcess::ConnectedAndOnline,
            ResultProcess::ConnectedAndOnline,
        ) {
            MapResult::Exception(_, _) => Err(anyhow::Error::msg("err")),
            _ => Ok(()),
        }
    }
    pub async fn send_disconnected_and_offline(&self) -> anyhow::Result<()> {
        match map_handlers!(
            &self,
            &DisconnectedAndOfflineEvent {},
            ModuleEventProcess::DisconnectAndOffline,
            ResultProcess::DisconnectAndOffline,
        ) {
            MapResult::Exception(_, _) => Err(anyhow::Error::msg("err")),
            _ => Ok(()),
        }
    }
}
