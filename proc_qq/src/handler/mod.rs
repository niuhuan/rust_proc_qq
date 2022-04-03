use crate::ModuleEventProcess::KickedOffline;
use async_trait::async_trait;
pub use events::*;
pub use processes::*;
use rs_qq::handler::{Handler, QEvent};
use std::sync::Arc;

mod events;
mod processes;

pub(crate) struct ClientHandler {
    pub(crate) modules: Arc<Vec<Module>>,
}

impl ClientHandler {}

enum MapResult<'a> {
    None,
    Process(&'a str, &'a str),
    Exception(&'a str, &'a str),
}

macro_rules! map_handlers {
    ($self:expr $(,$event:expr, $process:path)* $(,)?) => {{
        let mut result = MapResult::None;
        for m in $self.modules.as_ref() {
            for h in &m.handles {
                match &h.process {
                    $(
                    $process(e) => match e.handle(&$event).await {
                        Ok(b) => {
                            if b {
                                result = MapResult::Process(&m.id, &h.name);
                            }
                        }
                        Err(err) => {
                            tracing::error!(target = "proc_qq", " 出现错误 : {:?}", err);
                            result = MapResult::Exception(&m.id, &h.name);
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
                tracing::debug!(target = "proc_qq", "LOGIN : (UIN={})", event,);
                let _ = map_handlers!(
                    &self,
                    &LoginEvent { uin: event },
                    ModuleEventProcess::LoginEvent
                );
            }
            QEvent::GroupMessage(event) => {
                tracing::debug!(
                    target = "proc_qq",
                    "(GROUP={}, UIN={}) MESSAGE : {}",
                    event.message.group_code,
                    event.message.from_uin,
                    event.message.elements.to_string()
                );
                let me = MessageEvent::GroupMessage(event.clone());
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::GroupMessage,
                    &me,
                    ModuleEventProcess::Message,
                );
            }
            QEvent::FriendMessage(event) => {
                tracing::debug!(
                    target = "proc_qq",
                    "(UIN={}) MESSAGE : {}",
                    event.message.from_uin,
                    event.message.elements.to_string()
                );
                let me = MessageEvent::FriendMessage(event.clone());
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::FriendMessage,
                    &me,
                    ModuleEventProcess::Message,
                );
            }
            QEvent::TempMessage(event) => {
                tracing::debug!(
                    target = "proc_qq",
                    "(UIN={}) MESSAGE : {}",
                    event.message.from_uin,
                    event.message.elements.to_string()
                );
                let me = MessageEvent::TempMessage(event.clone());
                let _ = map_handlers!(
                    &self,
                    &event,
                    ModuleEventProcess::TempMessage,
                    &me,
                    ModuleEventProcess::Message,
                );
            }
            QEvent::GroupRequest(event) => {
                tracing::debug!(
                    target = "proc_qq",
                    "REQUEST (GROUP={}, UIN={}): {}",
                    event.request.group_code,
                    event.request.req_uin,
                    event.request.message,
                );
                let _ = map_handlers!(&self, &event, ModuleEventProcess::GroupRequest);
            }
            QEvent::FriendRequest(event) => {
                tracing::debug!(
                    target = "proc_qq",
                    "REQUEST (UIN={}): {}",
                    event.request.req_uin,
                    event.request.message
                );
                let _ = map_handlers!(&self, &event, ModuleEventProcess::FriendRequest);
            }
            QEvent::NewFriend(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::NewFriend);
            }
            QEvent::FriendPoke(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::FriendPoke);
            }
            QEvent::DeleteFriend(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::DeleteFriend);
            }
            QEvent::GroupMute(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::GroupMute);
            }
            QEvent::GroupLeave(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::GroupLeave);
            }
            QEvent::GroupNameUpdate(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::GroupNameUpdate);
            }
            QEvent::GroupMessageRecall(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::GroupMessageRecall);
            }
            QEvent::FriendMessageRecall(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::FriendMessageRecall);
            }
            QEvent::MSFOffline(event) => {
                let _ = map_handlers!(&self, &event, ModuleEventProcess::MSFOffline);
            }
            QEvent::KickedOffline(event) => {
                let _ = map_handlers!(&self, &event, KickedOffline);
            }
            _ => tracing::debug!(target = "proc_qq", "{:?}", e),
        }
    }
}

pub struct Module {
    pub id: String,
    pub name: String,
    pub handles: Vec<ModuleEventHandler>,
}
