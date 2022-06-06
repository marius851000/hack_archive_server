use actix_web::{cookie::Cookie, HttpResponseBuilder};

use crate::message::Messages;

pub trait HttpResponseBuilderExtension {
    fn with_messages(&mut self, messages: Messages) -> &mut Self;
}

impl HttpResponseBuilderExtension for HttpResponseBuilder {
    fn with_messages(&mut self, messages: Messages) -> &mut Self {
        //unwrap: Messages should never fail to serialize
        self.cookie(Cookie::build("messages", serde_json::to_string(&messages).unwrap()).finish())
    }
}
