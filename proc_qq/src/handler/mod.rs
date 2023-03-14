use std::sync::Arc;

use async_trait::async_trait;
use ricq::handler::{Handler, QEvent};
use ricq_core::msg::elem::RQElem;

pub use events::*;
pub use processes::*;
pub use results::*;

use crate::MessageContentTrait;

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
                        tracing::error!(target = "proc_qq", " 出现错误 : {:?}", err);
                        hand = true;
                    }
                },
                ResultProcess::OnlyResult(e) => match e.handle($event_result).await {
                    Ok(b) => {
                        hand = b;
                    }
                    Err(err) => {
                        tracing::error!(target = "proc_qq", " 出现错误 : {:?}", err);
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
                            tracing::error!(target = "proc_qq", " 出现错误 : {:?}", err);
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
                tracing::debug!(target = "proc_qq", "LOGIN : (UIN={})", event,);
                let _ = map_handlers!(
                    &self,
                    &LoginEvent { uin: event },
                    ModuleEventProcess::LoginEvent,
                    ResultProcess::LoginEvent,
                );
            }
            QEvent::GroupMessage(event) => {
                tracing::debug!(
                    target = "proc_qq",
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
                    target = "proc_qq",
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
                    target = "proc_qq",
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
                    target = "proc_qq",
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
                    target = "proc_qq",
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

#[derive(Clone, Debug)]
pub enum EventArg {
    All(Vec<EventArg>),
    Any(Vec<EventArg>),
    Not(Vec<EventArg>),
    Regexp(String),
    Eq(String),
    TrimRegexp(String),
    TrimEq(String),
}

#[derive(Clone)]
pub enum HandEvent<'a> {
    MessageEvent(&'a MessageEvent, String),
    FriendMessageEvent(&'a FriendMessageEvent, String),
    GroupMessageEvent(&'a GroupMessageEvent, String),
    GroupTempMessageEvent(&'a GroupTempMessageEvent, String),
}

impl HandEvent<'_> {
    pub fn content(&self) -> ::anyhow::Result<&'_ String> {
        Ok(match self {
            HandEvent::MessageEvent(_, content) => &content,
            HandEvent::FriendMessageEvent(_, content) => &content,
            HandEvent::GroupMessageEvent(_, content) => &content,
            HandEvent::GroupTempMessageEvent(_, content) => &content,
        })
    }
}

impl<'a> From<&'a MessageEvent> for HandEvent<'a> {
    fn from(value: &'a MessageEvent) -> Self {
        Self::MessageEvent(value, value.message_content())
    }
}

impl<'a> From<&'a FriendMessageEvent> for HandEvent<'a> {
    fn from(value: &'a FriendMessageEvent) -> Self {
        Self::FriendMessageEvent(value, value.message_content())
    }
}

impl<'a> From<&'a GroupMessageEvent> for HandEvent<'a> {
    fn from(value: &'a GroupMessageEvent) -> Self {
        Self::GroupMessageEvent(value, value.message_content())
    }
}

impl<'a> From<&'a GroupTempMessageEvent> for HandEvent<'a> {
    fn from(value: &'a GroupTempMessageEvent) -> Self {
        Self::GroupTempMessageEvent(value, value.message_content())
    }
}

pub fn match_event_args_all(args: Vec<EventArg>, event: HandEvent) -> ::anyhow::Result<bool> {
    for x in args {
        if !match_event_item(x, event.clone())? {
            return Ok(false);
        }
    }
    // 一个条件都没有认为是true
    Ok(true)
}

fn match_event_args_any(args: Vec<EventArg>, event: HandEvent) -> ::anyhow::Result<bool> {
    for x in args {
        if match_event_item(x, event.clone())? {
            return Ok(true);
        }
    }
    // 一个条件都没有认为是false
    Ok(false)
}

fn match_event_args_not(args: Vec<EventArg>, event: HandEvent) -> ::anyhow::Result<bool> {
    for x in args {
        if match_event_item(x, event.clone())? {
            return Ok(false);
        }
    }
    // 一个条件都没有认为是true
    Ok(true)
}

fn match_event_args_regexp(args: String, event: HandEvent) -> ::anyhow::Result<bool> {
    Ok(regex::Regex::new(args.as_str())?.is_match(event.content()?.as_str()))
}

fn match_event_args_eq(args: String, event: HandEvent) -> ::anyhow::Result<bool> {
    Ok(args.eq(event.content()?.as_str()))
}

fn match_event_args_trim_regexp(args: String, event: HandEvent) -> ::anyhow::Result<bool> {
    Ok(regex::Regex::new(args.as_str())?.is_match(event.content()?.trim()))
}

fn match_event_args_trim_eq(args: String, event: HandEvent) -> ::anyhow::Result<bool> {
    Ok(args.eq(event.content()?.trim()))
}

fn match_event_item(arg: EventArg, event: HandEvent) -> ::anyhow::Result<bool> {
    match arg {
        EventArg::All(v) => match_event_args_all(v, event.clone()),
        EventArg::Any(v) => match_event_args_any(v, event.clone()),
        EventArg::Not(v) => match_event_args_not(v, event.clone()),
        EventArg::Regexp(v) => match_event_args_regexp(v, event.clone()),
        EventArg::Eq(v) => match_event_args_eq(v, event.clone()),
        EventArg::TrimRegexp(v) => match_event_args_trim_regexp(v, event.clone()),
        EventArg::TrimEq(v) => match_event_args_trim_eq(v, event.clone()),
    }
}

//

pub struct CommandMatcher {
    pub idx: usize,
    pub elements: Vec<RQElem>,
    pub matching: String,
}

impl CommandMatcher {
    pub fn new(value: Vec<RQElem>) -> CommandMatcher {
        let mut matcher = CommandMatcher {
            idx: 0,
            elements: value,
            matching: String::new(),
        };
        matcher.push_text();
        matcher
    }

    pub fn push_text(&mut self) {
        loop {
            if self.idx >= self.elements.len() {
                break;
            }
            let ele: &RQElem = self.elements.get(self.idx).unwrap();
            match ele {
                RQElem::Text(st) => {
                    self.matching.push_str(st.content.as_str());
                    self.idx += 1;
                }
                _ => break,
            }
        }
        let build = self.matching.trim().to_string();
        self.matching = build;
    }

    pub fn match_command(&mut self, command_name: &str) -> bool {
        let sp_regexp = regex::Regex::new("\\s+").expect("proc_qq 正则错误");
        let mut sp = sp_regexp.split(self.matching.as_str());
        if let Some(first) = sp.next() {
            if command_name.eq(first) {
                self.matching = self.matching[first.len()..].trim().to_string();
                return true;
            }
        }
        return false;
    }

    pub fn not_blank(&self) -> bool {
        !self.matching.is_empty() || self.idx < self.elements.len()
    }
}

pub trait FromCommandMatcher: Sized {
    fn get(s: &mut CommandMatcher) -> Option<Self>;
}

#[inline]
pub fn matcher_get<F: Sized + FromCommandMatcher>(matcher: &mut CommandMatcher) -> Option<F> {
    F::get(matcher)
}

impl FromCommandMatcher for String {
    fn get(matcher: &mut CommandMatcher) -> Option<Self> {
        if matcher.matching.is_empty() {
            return None;
        }
        let sp_regexp = regex::Regex::new("\\s+").expect("proc_qq 正则错误");
        let mut sp = sp_regexp.split(matcher.matching.as_str());
        if let Some(first) = sp.next() {
            let result = Some(first.to_string());
            matcher.matching = matcher.matching[first.len()..].trim().to_string();
            return result;
        }
        None
    }
}

macro_rules! command_base_ty_supplier {
    ($ty:ty) => {
        impl FromCommandMatcher for $ty {
            fn get(matcher: &mut CommandMatcher) -> Option<$ty> {
                if matcher.matching.is_empty() {
                    return None;
                }
                let sp_regexp = regex::Regex::new("\\s+").expect("proc_qq 正则错误");
                let mut sp = sp_regexp.split(matcher.matching.as_str());
                if let Some(first) = sp.next() {
                    let result = match first.parse::<$ty>() {
                        Ok(value) => Some(value),
                        Err(_) => return None,
                    };
                    matcher.matching = matcher.matching[first.len()..].trim().to_string();
                    return result;
                }
                None
            }
        }
    };
}

command_base_ty_supplier!(i8);
command_base_ty_supplier!(u8);
command_base_ty_supplier!(i16);
command_base_ty_supplier!(u16);
command_base_ty_supplier!(i32);
command_base_ty_supplier!(u32);
command_base_ty_supplier!(i64);
command_base_ty_supplier!(u64);
command_base_ty_supplier!(i128);
command_base_ty_supplier!(u128);
command_base_ty_supplier!(isize);
command_base_ty_supplier!(usize);

macro_rules! command_rq_element_ty_supplier {
    ($ty:ty, $mat:path) => {
        impl FromCommandMatcher for $ty {
            fn get(matcher: &mut CommandMatcher) -> Option<Self> {
                if !matcher.matching.is_empty() {
                    return None;
                }
                if matcher.idx >= matcher.elements.len() {
                    return None;
                }
                if let $mat(item) = matcher.elements.get(matcher.idx).unwrap() {
                    let result = Some(item.clone());
                    matcher.idx += 1;
                    matcher.push_text();
                    return result;
                }
                None
            }
        }
    };
}

command_rq_element_ty_supplier!(ricq::msg::elem::At, RQElem::At);
command_rq_element_ty_supplier!(ricq::msg::elem::Face, RQElem::Face);
command_rq_element_ty_supplier!(ricq::msg::elem::MarketFace, RQElem::MarketFace);
command_rq_element_ty_supplier!(ricq::msg::elem::Dice, RQElem::Dice);
command_rq_element_ty_supplier!(ricq::msg::elem::FingerGuessing, RQElem::FingerGuessing);
command_rq_element_ty_supplier!(ricq::msg::elem::LightApp, RQElem::LightApp);
command_rq_element_ty_supplier!(ricq::msg::elem::RichMsg, RQElem::RichMsg);
command_rq_element_ty_supplier!(ricq::msg::elem::FriendImage, RQElem::FriendImage);
command_rq_element_ty_supplier!(ricq::msg::elem::GroupImage, RQElem::GroupImage);
command_rq_element_ty_supplier!(ricq::msg::elem::FlashImage, RQElem::FlashImage);
command_rq_element_ty_supplier!(ricq::msg::elem::VideoFile, RQElem::VideoFile);
