use crate::utils::CanReply;
use lazy_static::lazy_static;
use proc_qq::re_exports::ricq_core::msg::elem::RQElem;
use proc_qq::{
    event, module, ClientTrait, GroupTrait, MemberTrait, MessageContentTrait, MessageEvent, Module,
};
use regex::Regex;
use std::time::Duration;
static ID: &'static str = "group_admin";
static NAME: &'static str = "群管";

lazy_static! {
    static ref BAN_REGEXP: Regex =
        Regex::new("^(\\s+)?b(\\s+)?([0-9]{1,5})(\\s+)?([smhd]?)(\\s+)?").unwrap();
}

pub fn module() -> Module {
    module!(ID, NAME, on_message)
}

async fn not_in_group_and_reply(event: &MessageEvent) -> anyhow::Result<bool> {
    Ok(if !event.is_group_message() {
        event.reply_text("只能在群中使用").await?;
        true
    } else {
        false
    })
}

#[event]
async fn on_message(event: &MessageEvent) -> anyhow::Result<bool> {
    let content = event.message_content();
    if content == NAME {
        if not_in_group_and_reply(event).await? {
            return Ok(true);
        }
        event
            .reply_text(
                &("".to_owned()
                    + "b+禁言时间 @一个或多个人\n\n"
                    + "比如禁言张三12小时 : b12h @张三 \n\n"
                    + "比如禁言张三李四12天 : b12h @张三 @李四 \n\n"
                    + " s 秒, m 分, h 小时, d 天\n\n"
                    + "b0 则解除禁言"),
            )
            .await?;
        return Ok(true);
    }
    if !event.is_group_message() {
        return Ok(false);
    }
    let group_message = event.as_group_message()?;
    if BAN_REGEXP.is_match(&content) {
        // todo 缓存?
        let group = event
            .must_find_group(group_message.inner.group_code)
            .await?;
        let list = group_message
            .client
            .get_group_member_list(group.code, group.owner_uin)
            .await?;
        let call_member = list.must_find_member(event.from_uin()).await?;
        let bot_member = list.must_find_member(event.bot_uin().await).await?;
        if call_member.is_member() {
            group_message
                .reply_text("您必须是群主或管理员才能使用")
                .await?;
            return Ok(true);
        }
        if bot_member.is_member() {
            group_message
                .reply_text("机器人必须是群主或管理员才能使用")
                .await?;
            return Ok(true);
        }
        let e = BAN_REGEXP.captures_iter(&content).next().unwrap();
        let mut time = e.get(3).unwrap().as_str().parse::<u64>()?;
        time = match e.get(5).unwrap().as_str() {
            "m" => time * 60,
            "h" => time * 60 * 60,
            "d" => time * 60 * 60 * 24,
            _ => time,
        };
        if time >= 60 * 60 * 24 * 29 {
            group_message.reply_text("最多禁言29天").await?;
            return Ok(true);
        }
        for x in group_message.inner.elements.clone().into_iter() {
            match x {
                RQElem::At(id) => {
                    event
                        .client()
                        .group_mute(
                            group_message.inner.group_code,
                            id.target,
                            Duration::from_secs(time),
                        )
                        .await?;
                }
                _ => (),
            }
        }
        group_message.reply_text("OK").await?;
        return Ok(true);
    }
    Ok(false)
}
