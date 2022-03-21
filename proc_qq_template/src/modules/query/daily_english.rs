use crate::database::redis::{redis_get, redis_set};
use crate::utils::ffmpeg_cmd::{ffmpeg_convert, ffmpeg_run_version};
use crate::utils::local::{join_paths, template_dir};
use crate::utils::CanReply;
use anyhow::Context;
use proc_qq::re_exports::rs_qq::device::random_uuid;
use proc_qq::re_exports::rs_qq::structs::MessageReceipt;
use proc_qq::{event, module, MessageContentTrait, Module};
use proc_qq::{MessageEvent, MessageSendToSourceTrait};
use serde_derive::Deserialize;
use serde_derive::Serialize;
use silk_rs::encode_silk;
use std::time::Duration;

const ID: &str = "daily_english";
const NAME: &str = "每日英语";

pub(crate) fn module() -> Module {
    module!(ID, NAME, on_message)
}

#[event]
async fn on_message(message: &MessageEvent) -> anyhow::Result<bool> {
    if message.message_content().eq(NAME) {
        if message.is_temp_message() {
            message.reply_text("此功能不支持临时消息").await?;
            return Ok(true);
        }
        reply_daily_english(message).await?;
        return Ok(true);
    }
    Ok(false)
}

async fn reply_daily_english(message: &MessageEvent) -> anyhow::Result<()> {
    let today = chrono::Local::today();
    let today = today.format("%Y-%m-%d").to_string();
    let key = format!("DAILY_ENGLISH::{}", today);
    let mut daily: Option<DailyEnglish> = match redis_get::<String>(&key).await? {
        None => None,
        Some(context) => serde_json::from_str(&context)?,
    };
    if daily.is_none() {
        match ffmpeg_run_version() {
            Ok(_) => {}
            Err(_) => {
                message
                    .reply_text(
                        "机器人运行环境未安装ffmpeg命令, 无法使用此功能, 可联系机器人客服开启",
                    )
                    .await?;
                return anyhow::Result::Err(anyhow::Error::msg("FFMPEG 未安装"));
            }
        };
        daily = Some(load_form_network(&today).await?);
        let s = serde_json::to_string(&daily)?;
        redis_set(&key, s, 3600 * 24).await?;
    };
    let buff = daily.with_context(|| "wtf")?;
    message
        .send_audio_to_source(buff.buff, 1, Duration::from_secs(10))
        .await?;
    message
        .reply_text(&format!("{}\n\n{}", buff.content, buff.note))
        .await?;
    Ok(())
}

async fn load_form_network(day: &str) -> anyhow::Result<DailyEnglish> {
    let url = format!(
        "http://sentence.iciba.com/index.php?c=dailysentence&m=getdetail&title={}",
        day
    );
    let rsp: DailyEnglishRsp = serde_json::from_str(&reqwest::get(url).await?.text().await?)?;
    let buff = reqwest::get(rsp.tts).await?.bytes().await?.to_vec();
    let tmp_dir = template_dir();
    let uuid = random_uuid(&mut rand::thread_rng());
    let mp3 = join_paths(vec![&tmp_dir, &format!("{}.mp3", uuid)]);
    let pcm = join_paths(vec![&tmp_dir, &format!("{}.pcm", uuid)]);
    tokio::fs::write(&mp3, buff).await?;
    ffmpeg_convert(&mp3, &pcm)?;
    Ok(DailyEnglish {
        note: rsp.note,
        content: rsp.content,
        buff: encode_silk(tokio::fs::read(&pcm).await?, 24000, 48000, true)?,
    })
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyEnglishRsp {
    pub note: String,
    pub content: String,
    pub tts: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyEnglish {
    pub note: String,
    pub content: String,
    pub buff: Vec<u8>,
}
