use async_trait::async_trait;
use rs_qq::client::event::{GroupMessageEvent, PrivateMessageEvent, TempMessageEvent};
use rs_qq::handler::{Handler, QEvent};

pub struct ClientHandler {
    pub(crate) modules: Vec<Module>,
}

impl ClientHandler {}

macro_rules! map_handlers {
    ($self:expr,$event:expr,$process:path) => {
        for m in &$self.modules {
            for h in &m.handles {
                match &h.process {
                    $process(e) => {
                        match e.handle(&$event).await {
                            Ok(b) => {
                                if b {
                                    return;
                                }
                            }
                            Err(_) => {
                                // todo log
                                return;
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    };
}

#[async_trait]
impl Handler for ClientHandler {
    async fn handle(&self, e: QEvent) {
        match e {
            QEvent::GroupMessage(event) => {
                tracing::info!(target = "proc_qq", "MESSAGE : {:?}", event.message);
                map_handlers!(&self, &event, ModuleEventProcess::GroupMessage);
            }
            QEvent::PrivateMessage(event) => {
                tracing::info!(target = "proc_qq", "MESSAGE : {:?}", event.message);
                map_handlers!(&self, &event, ModuleEventProcess::PrivateMessage);
            }
            QEvent::TempMessage(event) => {
                tracing::info!(target = "proc_qq", "MESSAGE : {:?}", event.message);
                map_handlers!(&self, &event, ModuleEventProcess::TempMessage);
            }
            QEvent::GroupRequest(m) => {
                tracing::info!(
                    target = "proc_qq",
                    "REQUEST (GROUP={}, UIN={}): {}",
                    m.request.group_code,
                    m.request.req_uin,
                    m.request.message
                )
            }
            QEvent::FriendRequest(m) => {
                tracing::info!(
                    target = "proc_qq",
                    "REQUEST (UIN={}): {}",
                    m.request.req_uin,
                    m.request.message
                )
            }
            _ => tracing::info!(target = "proc_qq", "{:?}", e),
        }
    }
}

pub struct Module {
    pub id: String,
    pub handles: Vec<ModuleEventHandler>,
}

pub struct ModuleEventHandler {
    pub name: String,
    pub process: ModuleEventProcess,
}

pub enum ModuleEventProcess {
    GroupMessage(Box<dyn GroupMessageEventProcess>),
    PrivateMessage(Box<dyn PrivateMessageEventProcess>),
    TempMessage(Box<dyn TempMessageEventProcess>),
}

#[async_trait]
pub trait GroupMessageEventProcess: Sync + Send {
    async fn handle(&self, event: &GroupMessageEvent) -> anyhow::Result<bool>;
}

#[async_trait]
pub trait PrivateMessageEventProcess: Sync + Send {
    async fn handle(&self, event: &PrivateMessageEvent) -> anyhow::Result<bool>;
}

#[async_trait]
pub trait TempMessageEventProcess: Sync + Send {
    async fn handle(&self, event: &TempMessageEvent) -> anyhow::Result<bool>;
}
