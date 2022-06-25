use crate::database::mongo::{collection, db};
use crate::utils::CanReply;
use chrono::Duration;
use futures::stream::TryStreamExt;
use mongodb::bson::doc;
use mongodb::options::{IndexOptions, UpdateOptions};
use mongodb::{Collection, IndexModel};
use proc_qq::{event, module, MessageContentTrait, MessageEvent, Module};
use rand::distributions::{Distribution, Uniform};
use serde_derive::{Deserialize, Serialize};
use std::ops::Sub;

const ID: &str = "group_sign_in";
const NAME: &str = "群签到";

#[event]
async fn on_message(message: &MessageEvent) -> anyhow::Result<bool> {
    let content = message.message_content();
    if content.eq(NAME) {
        message
            .reply_text(
                r##" 群签到
在群中发出以下指令

 "签到" 获得金币, 每日一次
 "钱包" 查询金币剩余量
"##,
            )
            .await?;
        return Ok(true);
    }
    if message.is_group_message() {
        let group_code = message.as_group_message()?.inner.group_code;
        let uin = message.from_uin();
        let signs_coll: Collection<GroupSignIn> = collection("group_sign_in").await;
        if content.eq("签到") {
            let today = chrono::Local::today();
            let yesterday = today.sub(Duration::days(1));
            let today = today.format("%Y-%m-%d").to_string();
            let yesterday = yesterday.format("%Y-%m-%d").to_string();
            let pre: Option<GroupSignIn> = signs_coll
                .find_one(doc! {"group_code":group_code,"uin":uin}, None)
                .await?;
            let (up, result) = match pre {
                None => {
                    let up: i64 = 30;
                    let up: i64 = up + Uniform::<i64>::new(0, 100).sample(&mut rand::thread_rng());
                    (
                        up,
                        GroupSignIn {
                            group_code,
                            uin,
                            coins: up,
                            last_sign_in_date: today,
                            last_sign_in_count: 1,
                        },
                    )
                }
                Some(pre) => {
                    if pre.last_sign_in_date.eq(&today) {
                        message.reply_text("您今天已经签到过了").await?;
                        return Ok(true);
                    } else if pre.last_sign_in_date.eq(&yesterday) {
                        let up: i64 = 30;
                        let up: i64 =
                            up + Uniform::<i64>::new(0, 100).sample(&mut rand::thread_rng());
                        let up: i64 =
                            (up as f64 * pre.last_sign_in_count as f64 / 100.0 + up as f64) as i64;
                        (
                            up,
                            GroupSignIn {
                                group_code,
                                uin,
                                coins: pre.coins + up,
                                last_sign_in_date: today,
                                last_sign_in_count: pre.last_sign_in_count + 1,
                            },
                        )
                    } else {
                        let up = rand::random::<i64>() % 30;
                        (
                            up,
                            GroupSignIn {
                                group_code,
                                uin,
                                coins: pre.coins + up,
                                last_sign_in_date: today,
                                last_sign_in_count: 1,
                            },
                        )
                    }
                }
            };
            signs_coll
                .update_one(
                    doc! {"group_code":group_code,"uin":uin},
                    doc! {"$set":{
                        "coins": &result.coins,
                        "last_sign_in_date": &result.last_sign_in_date,
                        "last_sign_in_count": &result.last_sign_in_count,
                    }},
                    UpdateOptions::builder().upsert(true).build(),
                )
                .await?;
            message
                .reply_text(&format!(
                    "签到成功\n要坚持签到获得更多金币喔\n\n您一共连续签到{}天\n今天获得金币{}枚\n您一共有金币{}枚",
                    result.last_sign_in_count, up, result.coins
                ))
                .await?;
            return Ok(true);
        }
        if content.eq("钱包") {
            let pre: Option<GroupSignIn> = signs_coll
                .find_one(doc! {"group_code":group_code,"uin":uin}, None)
                .await?;
            match pre {
                None => {
                    message
                        .reply_text("您还没有开通群账户昂, 第一次签到自动开启")
                        .await?;
                }
                Some(pre) => {
                    message
                        .reply_text(&format!(
                            "您总共连续签到{}天\n最后一次签到时间为{}\n剩余金币{}枚",
                            pre.last_sign_in_count, pre.last_sign_in_date, pre.coins
                        ))
                        .await?;
                }
            }
            return Ok(true);
        }
    }
    Ok(false)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSignIn {
    pub group_code: i64,
    pub uin: i64,
    pub coins: i64,
    pub last_sign_in_date: String, // 2001-02-03
    pub last_sign_in_count: i64,
}

pub(crate) async fn init_data_base() -> anyhow::Result<()> {
    let db = db().await;
    let coll: Vec<_> = db
        .list_collections(
            doc! {
                "name": COLLECTION_NAME
            },
            None,
        )
        .await?
        .try_collect()
        .await?;
    if coll.len() == 0 {
        db.create_collection(COLLECTION_NAME, None).await?;
    }
    let signs_coll: Collection<GroupSignIn> = collection(COLLECTION_NAME).await;
    let names = signs_coll.list_index_names().await?;
    if names.contains(&"uk_group_and_uin".to_string()) {
        signs_coll
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "group_code": 1, "uin":1 })
                    .options(
                        IndexOptions::builder()
                            .name("uk_group_and_uin".to_string())
                            .unique(true)
                            .build(),
                    )
                    .build(),
                None,
            )
            .await?;
    }
    Ok(())
}

pub(crate) fn module() -> Module {
    module!(ID, NAME, on_message)
}

const COLLECTION_NAME: &str = "game.group_sign_in";
