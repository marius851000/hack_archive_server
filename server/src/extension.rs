use std::collections::HashMap;

use actix_web::{cookie::Cookie, HttpResponseBuilder};
use fluent_bundle::types::FluentValue;
use fluent_templates::{ArcLoader, Loader};
use log::warn;
use unic_langid::LanguageIdentifier;

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

//TODO: use a fallack (a.k.a english) language?
pub trait FluentLookupInfaillable: Loader {
    fn lookup_infaillable(&self, lang: &LanguageIdentifier, text_id: &str) -> String {
        self.lookup(lang, text_id)
            .unwrap_or_else(|| {
                warn!("Missing translation in {} without args: {}", lang, text_id);
                format!("<missing translation for {}>", text_id)
            })
    }

    fn lookup_with_args_infaillable<T: AsRef<str> + std::fmt::Debug>(
        &self,
        lang: &LanguageIdentifier,
        text_id: &str,
        args: &HashMap<T, FluentValue<'_>>,
    ) -> String {
        self.lookup_with_args(lang, text_id, args)
            .unwrap_or_else(|| {
                warn!("Missing translation in {} without args: {}", lang, text_id);
                format!("<missing translation for {} (args: {:?})>", text_id, args)
            })
    }
}

impl FluentLookupInfaillable for ArcLoader {}
