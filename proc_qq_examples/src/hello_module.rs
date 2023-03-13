pub use proc_qq::re_exports::async_trait::async_trait;
use proc_qq::re_exports::ricq::client::event::GroupMessageEvent;
use proc_qq::re_exports::ricq::msg::elem::At;
use proc_qq::{
    event, event_fn, module, LoginEvent, MessageChainParseTrait, MessageContentTrait, MessageEvent,
    MessageEventProcess, MessageSendToSourceTrait, Module,
};

/// 登录成功的时候调用
#[event]
async fn login(event: &LoginEvent) -> anyhow::Result<bool> {
    tracing::info!("正在登录 : {}", event.uin);
    Ok(false)
}

/// 任何消息都调用
#[event]
async fn print(event: &MessageEvent) -> anyhow::Result<bool> {
    let content = event.message_content();
    if content.eq("你好") {
        event
            .send_message_to_source("世界".parse_message_chain())
            .await?;
        Ok(true)
    } else if content.eq("RC") {
        event
            .send_message_to_source("NB".parse_message_chain())
            .await?;
        Ok(true)
    } else if content.eq("EX") {
        Err(anyhow::Error::msg("Text exception"))
    } else {
        Ok(false)
    }
}

/// 群消息时调用
#[event]
async fn group_hello(_: &GroupMessageEvent) -> anyhow::Result<bool> {
    Ok(false)
}

/// 自定义Handler

struct OnMessage;

#[async_trait]
impl MessageEventProcess for OnMessage {
    async fn handle(&self, event: &MessageEvent) -> anyhow::Result<bool> {
        self.do_some(event).await?;
        Ok(true)
    }
}

impl OnMessage {
    async fn do_some(&self, _: &MessageEvent) -> anyhow::Result<()> {
        Ok(())
    }
}

/// 使用正则
#[event(regexp = "^(\\s+)?你很好(\\s+)?$")]
async fn handle(event: &MessageEvent) -> anyhow::Result<bool> {
    event
        .send_message_to_source("你也很好".parse_message_chain())
        .await?;
    Ok(true)
}

/// 多个规则, 支持 trim_regexp trim_eq all any not regexp eq
/// 支持嵌套使用 all(not(any(),eq = ""))
#[event(trim_regexp = "^a([\\S\\s]+)?$", trim_regexp = "^([\\S\\s]+)?b$")]
async fn handle2(event: &MessageEvent) -> anyhow::Result<bool> {
    event
        .send_message_to_source("a开头且b结束".parse_message_chain())
        .await?;
    Ok(true)
}

/// 解决调用函数生命周期问题, 使用self调用 event_fn

#[event]
async fn handle3(message: &MessageEvent) -> anyhow::Result<bool> {
    self.handle3_add(message).await;
    Ok(false)
}

#[event]
async fn handle4(message: &MessageEvent) -> anyhow::Result<bool> {
    self.handle3_add(message).await;
    Ok(false)
}

#[event_fn(handle3, handle4)]
async fn handle3_add(message: &MessageEvent) {
    println!("{}", message.message_content());
}

/// bot_command

#[event(bot_command = "/ban {user} {time}")]
async fn handle5(_message: &MessageEvent, user: String, time: i64) -> anyhow::Result<bool> {
    println!("handle5。user : {user} , time : {time} ");
    Ok(true)
}

#[event(bot_command = "/ban {user} {time}")]
async fn handle6(_message: &MessageEvent, user: At, time: i64) -> anyhow::Result<bool> {
    println!("handle6。 user : {user} , time : {time} ");
    Ok(true)
}

/// module

pub fn module() -> Module {
    module!(
        "hello",
        "你好",
        login,
        print,
        group_hello,
        handle,
        handle2,
        handle3,
        handle4,
        handle5,
        handle6,
    )
}
