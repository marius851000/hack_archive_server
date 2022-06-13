use std::{collections::HashMap, convert::Infallible, future::Future, pin::Pin, sync::Arc};

use actix_web::{web::Data, FromRequest};
use database::model::MajorityToken;
use fluent_templates::{fluent_bundle::FluentValue, LanguageIdentifier, Loader};
use qstring::QString;
use unic_langid::langid;

use crate::{
    message::{MessageKind, Messages},
    AppData,
};

pub struct RequestData {
    pub majority_token: Option<MajorityToken>,
    pub have_access_to_major_only_content: bool,
    pub can_certify: bool,
    pub messages: Messages,
    pub language: LanguageIdentifier,
    pub path: String,
    pub app_data: Arc<AppData>,
}

impl FromRequest for RequestData {
    type Error = Infallible;

    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let mut messages = Messages::default();

        let mut path = req.path().to_string();
        if path.starts_with('/') {
            path = path.chars().skip(1).collect();
        }
        let app_data = req.app_data::<Data<AppData>>().unwrap().clone();

        let query_string = QString::from(req.query_string());
        let language = query_string
            .get("lang")
            .unwrap_or("en")
            .parse()
            .unwrap_or(langid!("en"));

        if let Some(cookie_messages) = req.cookie("messages") {
            if !cookie_messages.value().is_empty() {
                match serde_json::from_str::<Messages>(cookie_messages.value()) {
                    Ok(messages_in_cookie) => messages = messages_in_cookie,
                    Err(_) => messages.add_message_from_string(
                        app_data
                            .locales
                            .lookup(&language, "error-notication-cant-be-read"),
                        MessageKind::Error,
                    ),
                };
            };
        }

        if query_string.get("redirect_url_error").is_some() {
            messages.add_message_from_string(
                app_data.locales.lookup(&language, "message-error-redirect"),
                MessageKind::Error,
            )
        }

        let majority_token_cookie = req.cookie("majority_token");

        Box::pin(async move {
            let mut majority_token: Option<String> =
                majority_token_cookie.map(|cookie| cookie.value().to_string());

            if majority_token.as_deref() == Some("") {
                majority_token = None;
            };

            let (majority_token, have_access_to_major_only_content, can_certify) =
                if let Some(token) = majority_token {
                    app_data
                        .check_validity_of_majority_token(&token, &mut messages, &language)
                        .await
                } else {
                    (None, false, false)
                };

            Ok(Self {
                majority_token,
                have_access_to_major_only_content,
                can_certify,
                messages,
                language,
                path,
                app_data: app_data.into_inner(),
            })
        })
    }
}

impl RequestData {
    pub fn lookup(&self, text_id: &str) -> String {
        self.app_data.locales.lookup(&self.language, text_id)
    }

    pub fn lookup_with_args(&self, text_id: &str, args: &HashMap<&str, FluentValue>) -> String {
        self.app_data
            .locales
            .lookup_with_args(&self.language, text_id, args)
    }
}
