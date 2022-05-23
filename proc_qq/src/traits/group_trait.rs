use async_trait::async_trait;
use ricq_core::structs::GroupMemberInfo;
use ricq_core::{RQError, RQResult};

#[async_trait]
pub trait GroupTrait {
    async fn must_find_member(&self, uid: i64) -> RQResult<GroupMemberInfo>;
}

#[async_trait]
impl GroupTrait for Vec<GroupMemberInfo> {
    async fn must_find_member(&self, uin: i64) -> RQResult<GroupMemberInfo> {
        for x in self.iter() {
            if x.uin == uin {
                return RQResult::Ok(x.clone());
            }
        }
        return RQResult::Err(RQError::Other(format!("Member nor found : (uin={})", uin)));
    }
}
