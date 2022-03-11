use maud::{html, Markup};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Error,
    Success,
}

pub struct Message {
    kind: MessageKind,
    value: Markup,
}

impl Message {
    pub fn kind(&self) -> MessageKind {
        self.kind
    }

    pub fn value(&self) -> &Markup {
        &self.value
    }

    pub fn from_string(text: String, kind: MessageKind) -> Self {
        Self {
            kind,
            value: html!(
                p { (text) }
            ),
        }
    }

    pub fn from_markup(markup: Markup, kind: MessageKind) -> Self {
        Self {
            kind,
            value: markup,
        }
    }
}

#[derive(Default)]
pub struct Messages {
    messages: Vec<Message>,
}

impl Messages {
    pub fn messages(&self) -> &Vec<Message> {
        &self.messages
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn have_error(&self) -> bool {
        let mut have_error = false;
        for message in &self.messages {
            have_error |= message.kind() == MessageKind::Error
        }
        have_error
    }

    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    pub fn add_message_from_string(&mut self, text: String, kind: MessageKind) {
        self.add_message(Message::from_string(text, kind));
    }

    pub fn add_message_from_markup(&mut self, markup: Markup, kind: MessageKind) {
        self.add_message(Message::from_markup(markup, kind));
    }
}
