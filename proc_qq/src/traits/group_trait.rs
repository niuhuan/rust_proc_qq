use async_trait::async_trait;
use ricq::structs::Group;
use ricq_core::structs::GroupMemberInfo;
use ricq_core::{RQError, RQResult};

#[async_trait]
pub trait GroupTrait {
    async fn must_find_member(&self, uid: i64) -> RQResult<GroupMemberInfo>;
}

#[async_trait]
impl GroupTrait for Group {
    async fn must_find_member(&self, uin: i64) -> RQResult<GroupMemberInfo> {
        let lock = self.members.read().await;
        for x in lock.iter() {
            if x.uin == uin {
                return RQResult::Ok(x.clone());
            }
        }
        return RQResult::Err(RQError::Other(format!(
            "Member nor found : (GROUP_CODE={},uin={})",
            self.info.code, uin
        )));
    }
}
