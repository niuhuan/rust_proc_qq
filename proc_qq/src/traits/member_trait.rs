use rq_engine::structs::{GroupMemberInfo, GroupMemberPermission};

pub trait MemberTrait {
    fn is_member(&self) -> bool;
}

impl MemberTrait for GroupMemberInfo {
    fn is_member(&self) -> bool {
        match &self.permission {
            GroupMemberPermission::Member => true,
            _ => false,
        }
    }
}
