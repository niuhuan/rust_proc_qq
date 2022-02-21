use async_trait::async_trait;
use rq_engine::msg::MessageChain;
use rq_engine::structs::MessageReceipt;
use rq_engine::{RQError, RQResult};

use crate::{MessageSource, MessageTrait};

#[async_trait]
pub trait ClientTrait: Send + Sync {
    async fn send_message_to_source(
        &self,
        message: &impl MessageTrait,
        message_chain: MessageChain,
    ) -> RQResult<MessageReceipt>;
}

#[async_trait]
impl ClientTrait for rs_qq::Client {
    async fn send_message_to_source(
        &self,
        message: &impl MessageTrait,
        message_chain: MessageChain,
    ) -> RQResult<MessageReceipt> {
        match message.message_source() {
            MessageSource::Group(group_code, _) => {
                self.send_group_message(group_code, message_chain).await
            }
            MessageSource::Private(uin) => self.send_private_message(uin, message_chain).await,
            MessageSource::Temp(group_code, uin) => {
                if let Some(group_code) = group_code {
                    match self.send_temp_message(group_code, uin, message_chain).await {
                        Ok(_) => RQResult::Ok(MessageReceipt::default()),
                        Err(err) => RQResult::Err(err),
                    }
                } else {
                    RQResult::Err(RQError::Other("不存在GroupCode".to_owned()))
                }
            }
            MessageSource::Unsupported => RQResult::Err(RQError::Other("不支持的类型".to_owned())),
        }
    }
}

#[async_trait]
impl ClientTrait for crate::Client {
    async fn send_message_to_source(
        &self,
        message: &impl MessageTrait,
        message_chain: MessageChain,
    ) -> RQResult<MessageReceipt> {
        self.rq_client
            .send_message_to_source(message, message_chain)
            .await
    }
}
