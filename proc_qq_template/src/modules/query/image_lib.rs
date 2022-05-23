use crate::utils::CanReply;
use anyhow::Context;
use proc_qq::re_exports::ricq::msg::MessageChain;
use proc_qq::re_exports::{bytes, reqwest};
use proc_qq::{
    event, module, MessageChainAppendTrait, MessageContentTrait, MessageEvent,
    MessageSendToSourceTrait, Module, TextEleParseTrait,
};
use regex::Regex;

static ID: &'static str = "image_lib";
static NAME: &'static str = "图库";
static MENU: &'static str = "图库 (请直接回复功能名) : \n ❤️ 随机老婆\n ❤️ 动漫壁纸";
static UA: &'static str = "Mozilla/5.0 (Linux; Android 6.0; Nexus 5 Build/MRA58N) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/98.0.4758.80 Mobile Safari/537.36";

pub fn module() -> Module {
    module!(ID, NAME, on_message)
}

fn no_temp_message() -> MessageChain {
    MessageChain::default().append("临时会话不能使用此功能奥".parse_text())
}

#[event]
async fn on_message(event: &MessageEvent) -> anyhow::Result<bool> {
    let content = event.message_content();
    if content.eq(NAME) {
        if event.is_temp_message() {
            event.send_message_to_source(no_temp_message()).await?;
        } else {
            event
                .send_message_to_source(event.make_reply_chain().await.append(MENU.parse_text()))
                .await?;
        }
        Ok(true)
    } else if content.eq("随机老婆") {
        if event.is_temp_message() {
            event.send_message_to_source(no_temp_message()).await?;
            return Ok(true);
        }
        let img = get_laopo_img().await?.to_vec();
        let img = event.upload_image_to_source(img).await?;
        event
            .send_message_to_source(event.make_reply_chain().await.append(img))
            .await?;
        Ok(true)
    } else if content.eq("动漫壁纸") {
        if event.is_temp_message() {
            event.send_message_to_source(no_temp_message()).await?;
            return Ok(true);
        }
        let img = get_dongman_img().await?.to_vec();
        let img = event.upload_image_to_source(img).await?;
        event
            .send_message_to_source(event.make_reply_chain().await.append(img))
            .await?;
        Ok(true)
    } else {
        Ok(false)
    }
}

async fn get_laopo_img() -> anyhow::Result<bytes::Bytes> {
    let text = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()?
        .request(reqwest::Method::GET, "https://img.xjh.me/random_img.php")
        .header("User-Agent", UA)
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    let regex = Regex::new("src=\"//(.+\\.jpg)\"")?;
    let mt = regex
        .captures_iter(&text)
        .next()
        .with_context(|| "未能成功取得图片")?;
    let url = format!("https://{}", mt.get(1).unwrap().as_str());
    let buff = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()?
        .request(reqwest::Method::GET, url.as_str())
        .header("User-Agent", UA)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    Ok(buff)
}

async fn get_dongman_img() -> anyhow::Result<bytes::Bytes> {
    let buff = reqwest::ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()?
        .request(
            reqwest::Method::GET,
            "http://api.molure.cn/sjbz/api.php?lx=dongman",
        )
        .header("User-Agent", UA)
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    Ok(buff)
}
