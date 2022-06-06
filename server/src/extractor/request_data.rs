use std::{collections::HashMap, convert::Infallible, future::Future, pin::Pin, sync::Arc};

use actix_web::{web::Data, FromRequest};
use database::{model::MajorityToken, HackClient};
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
    pub majority_cookie_to_set: Option<String>,
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
                            .lookup(&language, "error-notication-cant-be-read")
                            .to_string(),
                        MessageKind::Error,
                    ),
                };
            };
        }

        if !app_data.use_majority_token {
            return Box::pin(async move {
                Ok(RequestData {
                    majority_token: None,
                    have_access_to_major_only_content: false,
                    can_certify: false,
                    messages,
                    majority_cookie_to_set: None,
                    language,
                    path,
                    app_data: app_data.into_inner(),
                })
            });
        }

        //TODO: put the majority token in post
        let cookie = req.cookie("majority_token");

        let mut parameter_majority_token = query_string
            .get("majority_token")
            .map(|code| code.to_string());

        if parameter_majority_token.as_deref() == Some("") {
            parameter_majority_token = None;
        }

        Box::pin(async move {
            let parameter_majority_token: Option<String> = parameter_majority_token;
            let mut majority_token: Option<String> =
                if let Some(token) = parameter_majority_token.as_ref() {
                    Some(token.to_string())
                } else {
                    cookie.map(|cookie| cookie.value().to_string())
                };

            if majority_token.as_deref() == Some("") {
                majority_token = None;
            };

            let (majority_token, have_access_to_major_only_content, can_certify) =
                if let Some(token) = majority_token {
                    app_data
                        .check_validity_of_majority_token(&token, &mut messages)
                        .await
                } else {
                    (None, false, false)
                };

            Ok(Self {
                majority_token,
                have_access_to_major_only_content,
                can_certify,
                messages,
                majority_cookie_to_set: if have_access_to_major_only_content {
                    parameter_majority_token
                } else {
                    None
                },
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
