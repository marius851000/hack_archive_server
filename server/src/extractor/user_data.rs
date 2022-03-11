use std::{future::Future, pin::Pin};

use actix_web::{web::Data, FromRequest, HttpMessage};
use database::{MajorityCheck, MongoDriver};
use qstring::QString;

use crate::message::{MessageKind, Messages};

pub struct UserData {
    pub majority: Option<MajorityCheck>,
    pub have_access_to_major_only_content: bool,
    pub messages: Messages,
    pub majority_cookie_to_set: Option<String>,
}

impl FromRequest for UserData {
    type Error = ();

    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    type Config = ();

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        //TODO: maybe put the majority token in post
        //TODO: button to remove the token
        let cookie = req.cookie("majority_token");
        let mongodriver = req.app_data::<Data<MongoDriver>>().unwrap().clone();

        let query_string = QString::from(req.query_string());
        let parameter_majority_token = query_string
            .get("majority_code")
            .map(|code| code.to_string());

        Box::pin(async move {
            let mut messages = Messages::default();
            let parameter_majority_token: Option<String> = parameter_majority_token;
            let majority_token: Option<String> =
                if let Some(token) = parameter_majority_token.as_ref() {
                    Some(token.to_string())
                } else {
                    cookie.map(|cookie| cookie.value().to_string())
                };

            let (majority, have_access_to_major_only_content) = if let Some(majority_token) =
                majority_token.as_ref()
            {
                match mongodriver
                    .get_majority_check_by_token(majority_token)
                    .await
                {
                    Ok(majority) => {
                        if let Some(majority) = majority {
                            let have_access =
                                !majority.sources.is_empty() || !majority.require_source;
                            if have_access {
                                messages.add_message_from_string(
                                    "The provided majority token is valid".into(),
                                    MessageKind::Error,
                                );
                            } else {
                                messages.add_message_from_string(
                                "The current token doesn't allow access to mature content. It may have been revoked. Please contact the administrator.".into(),
                                MessageKind::Error
                                );
                            }
                            (Some(majority), have_access)
                        } else {
                            messages.add_message_from_string(
                                "The token provided doesn't exist in the database.".into(),
                                MessageKind::Error,
                            );
                            (None, false)
                        }
                    }
                    Err(e) => {
                        println!(
                            "an error occured while checking the majority of the user : {:?}",
                            e
                        );
                        messages.add_message_from_string(
                        "An (internal ?) error occured while checking wheter you have access to mature content or not. You won't have access to this content. If you need help, contact the author, marius.".into(),
                        MessageKind::Error
                        );
                        (None, false)
                    }
                }
            } else {
                (None, false)
            };
            Ok(Self {
                majority,
                have_access_to_major_only_content,
                messages,
                majority_cookie_to_set: if have_access_to_major_only_content {
                    parameter_majority_token
                } else {
                    None
                },
            })
        })
    }
}
