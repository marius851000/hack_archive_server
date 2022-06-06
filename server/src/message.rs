use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Debug)]
pub enum MessageKind {
    Error,
    Success,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    kind: MessageKind,
    value: String,
}

impl Message {
    pub fn kind(&self) -> MessageKind {
        self.kind
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    pub fn from_string(text: String, kind: MessageKind) -> Self {
        Self { kind, value: text }
    }
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct Messages(Vec<Message>);

impl Messages {
    pub fn create_with_message(text: String, kind: MessageKind) -> Self {
        Self(vec![Message::from_string(text, kind)])
    }

    pub fn messages(&self) -> &Vec<Message> {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn have_error(&self) -> bool {
        let mut have_error = false;
        for message in &self.0 {
            have_error |= message.kind() == MessageKind::Error
        }
        have_error
    }

    pub fn add_message(&mut self, message: Message) {
        self.0.push(message);
    }

    pub fn add_message_from_string(&mut self, text: String, kind: MessageKind) {
        self.add_message(Message::from_string(text, kind));
    }
}
