use ricq_core::msg::MessageChain;
use ricq_core::pb::msg::elem::Elem;

pub trait MessageChainTrait {
    fn append<S: Into<Vec<Elem>>>(self, elem: S) -> Self;
}

impl MessageChainTrait for MessageChain {
    fn append<S: Into<Vec<Elem>>>(self, elem: S) -> Self {
        let mut chain = self;
        chain.push(elem);
        chain
    }
}
