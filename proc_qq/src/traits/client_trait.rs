use async_trait::async_trait;
use ricq::structs::GroupInfo;
use ricq_core::common::group_code2uin;
use ricq_core::msg::MessageChain;
use ricq_core::structs::MessageReceipt;
use ricq_core::{RQError, RQResult};

use crate::{MessageTarget, MessageTargetTrait};

#[async_trait]
pub trait ClientTrait: Send + Sync {
    async fn send_message_to_target<S: Into<MessageChain> + Send + Sync>(
        &self,
        source: &impl MessageTargetTrait,
        message: S,
    ) -> RQResult<MessageReceipt>;
    async fn must_find_group(&self, group_code: i64) -> RQResult<GroupInfo>;
    async fn bot_uin(&self) -> i64;
}

#[async_trait]
impl ClientTrait for ricq::Client {
    async fn send_message_to_target<S: Into<MessageChain> + Send + Sync>(
        &self,
        source: &impl MessageTargetTrait,
        message: S,
    ) -> RQResult<MessageReceipt> {
        let message = message.into();
        match source.target() {
            MessageTarget::Group(group_code, _) => {
                self.send_group_message(group_code, message).await
            }
            MessageTarget::Private(uin) => self.send_friend_message(uin, message).await,
            MessageTarget::GroupTemp(group_code, uin) => {
                match self
                    .send_message(
                        ricq_core::pb::msg::routing_head::RoutingHead::GrpTmp(
                            ricq_core::pb::msg::GrpTmp {
                                group_uin: Some(group_code2uin(group_code)),
                                to_uin: Some(uin),
                            },
                        ),
                        message,
                        None,
                    )
                    .await
                {
                    Ok(_) => RQResult::Ok(MessageReceipt::default()),
                    Err(err) => RQResult::Err(err),
                }
            }
        }
    }
    async fn must_find_group(&self, group_code: i64) -> RQResult<GroupInfo> {
        let group = self.get_group_info(group_code).await?;
        match group {
            Some(group) => RQResult::Ok(group),
            None => RQResult::Err(RQError::Other(format!("Group not found : {}", group_code))),
        }
    }

    async fn bot_uin(&self) -> i64 {
        self.uin().await
    }
}

#[async_trait]
impl ClientTrait for crate::Client {
    async fn send_message_to_target<S: Into<MessageChain> + Send + Sync>(
        &self,
        source: &impl MessageTargetTrait,
        message: S,
    ) -> RQResult<MessageReceipt> {
        self.rq_client
            .send_message_to_target(source, message.into())
            .await
    }

    async fn must_find_group(&self, group_code: i64) -> RQResult<GroupInfo> {
        self.rq_client.must_find_group(group_code).await
    }

    async fn bot_uin(&self) -> i64 {
        self.rq_client.bot_uin().await
    }
}
