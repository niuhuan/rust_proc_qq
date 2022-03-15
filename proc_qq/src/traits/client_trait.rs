use async_trait::async_trait;
use rq_engine::msg::MessageChain;
use rq_engine::structs::MessageReceipt;
use rq_engine::{RQError, RQResult};
use rs_qq::structs::Group;
use std::sync::Arc;

use crate::{MessageSource, MessageSourceTrait};

#[async_trait]
pub trait ClientTrait: Send + Sync {
    async fn send_message_to_source<S: Into<MessageChain> + Send + Sync>(
        &self,
        source: &impl MessageSourceTrait,
        message: S,
    ) -> RQResult<MessageReceipt>;
    async fn must_find_group(&self, group_code: i64, auto_reload: bool) -> RQResult<Arc<Group>>;
}

#[async_trait]
impl ClientTrait for rs_qq::Client {
    async fn send_message_to_source<S: Into<MessageChain> + Send + Sync>(
        &self,
        source: &impl MessageSourceTrait,
        message: S,
    ) -> RQResult<MessageReceipt> {
        let message = message.into();
        match source.message_source() {
            MessageSource::Group(group_code, _) => {
                self.send_group_message(group_code, message).await
            }
            MessageSource::Private(uin) => self.send_private_message(uin, message).await,
            MessageSource::Temp(group_code, uin) => {
                if let Some(group_code) = group_code {
                    match self.send_temp_message(group_code, uin, message).await {
                        Ok(_) => RQResult::Ok(MessageReceipt::default()),
                        Err(err) => RQResult::Err(err),
                    }
                } else {
                    RQResult::Err(RQError::Other("不存在GroupCode".to_owned()))
                }
            }
        }
    }
    async fn must_find_group(&self, group_code: i64, auto_reload: bool) -> RQResult<Arc<Group>> {
        let group = self.find_group(group_code, auto_reload).await;
        match group {
            Some(group) => RQResult::Ok(group),
            None => RQResult::Err(RQError::Other(format!("Group not found : {}", group_code))),
        }
    }
}

#[async_trait]
impl ClientTrait for crate::Client {
    async fn send_message_to_source<S: Into<MessageChain> + Send + Sync>(
        &self,
        source: &impl MessageSourceTrait,
        message: S,
    ) -> RQResult<MessageReceipt> {
        self.rq_client
            .send_message_to_source(source, message.into())
            .await
    }

    async fn must_find_group(&self, group_code: i64, auto_reload: bool) -> RQResult<Arc<Group>> {
        self.rq_client
            .must_find_group(group_code, auto_reload)
            .await
    }
}
