use async_trait::async_trait;
use rs_qq::client::event::GroupMessageEvent;
use rs_qq::handler::{Handler, QEvent};

pub struct ClientHandler {
    pub(crate) modules: Vec<Module>,
}

impl ClientHandler {}

#[async_trait]
impl Handler for ClientHandler {
    async fn handle(&self, e: QEvent) {
        match e {
            QEvent::GroupMessage(m) => {
                tracing::info!(
                    target = "rs_qq",
                    "MESSAGE (GROUP={}): {}",
                    m.message.group_code,
                    m.message.elements
                )
            }
            QEvent::PrivateMessage(m) => {
                tracing::info!(
                    target = "rs_qq",
                    "MESSAGE (FRIEND={}): {}",
                    m.message.from_uin,
                    m.message.elements
                )
            }
            QEvent::TempMessage(m) => {
                tracing::info!(
                    target = "rs_qq",
                    "MESSAGE (TEMP={}): {}",
                    m.message.from_uin,
                    m.message.elements
                )
            }
            QEvent::GroupRequest(m) => {
                tracing::info!(
                    target = "rs_qq",
                    "REQUEST (GROUP={}, UIN={}): {}",
                    m.request.group_code,
                    m.request.req_uin,
                    m.request.message
                )
            }
            QEvent::FriendRequest(m) => {
                tracing::info!(
                    target = "rs_qq",
                    "REQUEST (UIN={}): {}",
                    m.request.req_uin,
                    m.request.message
                )
            }
            _ => tracing::info!(target = "rs_qq", "{:?}", e),
        }
    }
}

pub struct Module {
    pub id: String,
    pub handles: Vec<ModelEventHandle>,
}

pub enum ModelEventHandle {
    GroupMessageEventHandle(Box<dyn ModuleGroupMessageEventHandle>),
}

#[async_trait]
pub trait ModuleGroupMessageEventHandle: Sync + Send {
    async fn handle(&self, event: &GroupMessageEvent);
}
