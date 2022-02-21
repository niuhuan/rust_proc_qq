use rq_engine::msg::MessageChain;
use rq_engine::structs::{GroupMessage, PrivateMessage, TempMessage};
use rs_qq::client::event::{GroupMessageEvent, PrivateMessageEvent, TempMessageEvent};

pub enum MessageSource {
    // Group(group_code,uin)
    Group(i64, i64),
    // Private(uin)
    Private(i64),
    // Temp(group_code,uin)
    Temp(Option<i64>, i64),
    // Unsupported
    Unsupported,
}

pub trait MessageTrait: Send + Sync {
    fn message_source(&self) -> MessageSource;
    fn message_content(&self) -> String;
}

impl MessageTrait for MessageChain {
    fn message_source(&self) -> MessageSource {
        MessageSource::Unsupported
    }

    fn message_content(&self) -> String {
        self.to_string()
    }
}

impl MessageTrait for GroupMessage {
    fn message_source(&self) -> MessageSource {
        MessageSource::Group(self.group_code, self.from_uin)
    }

    fn message_content(&self) -> String {
        self.elements.message_content()
    }
}

impl MessageTrait for GroupMessageEvent {
    fn message_source(&self) -> MessageSource {
        self.message.message_source()
    }

    fn message_content(&self) -> String {
        self.message.message_content()
    }
}

impl MessageTrait for PrivateMessage {
    fn message_source(&self) -> MessageSource {
        MessageSource::Private(self.from_uin)
    }

    fn message_content(&self) -> String {
        self.elements.message_content()
    }
}

impl MessageTrait for PrivateMessageEvent {
    fn message_source(&self) -> MessageSource {
        self.message.message_source()
    }

    fn message_content(&self) -> String {
        self.message.message_content()
    }
}

impl MessageTrait for TempMessage {
    fn message_source(&self) -> MessageSource {
        MessageSource::Temp(self.group_code, self.from_uin)
    }

    fn message_content(&self) -> String {
        self.elements.message_content()
    }
}

impl MessageTrait for TempMessageEvent {
    fn message_source(&self) -> MessageSource {
        self.message.message_source()
    }

    fn message_content(&self) -> String {
        self.message.message_content()
    }
}
