use std::{convert::Infallible, future::Future, pin::Pin, sync::Arc};

use actix_web::{web::Data, FromRequest};
use database::{model::MajorityToken, HackClient};
use qstring::QString;

use crate::{
    message::{MessageKind, Messages},
    AppData,
};

pub struct RequestData {
    pub majority: Option<MajorityToken>,
    pub have_access_to_major_only_content: bool,
    pub can_certify: bool,
    pub messages: Messages,
    pub majority_cookie_to_set: Option<String>,
}

impl FromRequest for RequestData {
    type Error = Infallible;

    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let app_data = req.app_data::<Data<Arc<AppData>>>().unwrap().clone();
        if !app_data.use_majority_token {
            return Box::pin(async move {
                Ok(RequestData {
                    majority: None,
                    have_access_to_major_only_content: false,
                    can_certify: false,
                    messages: Messages::default(),
                    majority_cookie_to_set: None,
                })
            });
        }

        //TODO: put the majority token in post
        let cookie = req.cookie("majority_token");
        let hackclient = req.app_data::<Data<HackClient>>().unwrap().clone();

        let query_string = QString::from(req.query_string());
        let mut parameter_majority_token = query_string
            .get("majority_token")
            .map(|code| code.to_string());

        if parameter_majority_token.as_ref().map(|x| x.as_str()) == Some("") {
            parameter_majority_token = None;
        }

        let should_disconnect = query_string.get("disconnect_majority_token").is_some();

        Box::pin(async move {
            let mut messages = Messages::default();
            if should_disconnect {
                return Ok(Self {
                    majority: None,
                    have_access_to_major_only_content: false,
                    can_certify: false,
                    messages,
                    majority_cookie_to_set: Some(String::new()),
                });
            };
            let parameter_majority_token: Option<String> = parameter_majority_token;
            let majority_token: Option<String> =
                if let Some(token) = parameter_majority_token.as_ref() {
                    Some(token.to_string())
                } else {
                    cookie.map(|cookie| cookie.value().to_string())
                };

            let (majority, have_access_to_major_only_content, can_certify) = if let Some(
                majority_token,
            ) =
                majority_token.as_ref()
            {
                match hackclient.get_majority_token(majority_token).await {
                    Ok(Some(majority)) => {
                        if majority.admin_flags.get().revoked {
                            messages.add_message_from_string(
                                "The provided majority token has been revoked by an administrator"
                                    .into(),
                                MessageKind::Error,
                            );
                            (Some(majority), false, false)
                        } else {
                            let can_certify = majority.admin_flags.get().can_certify;
                            (Some(majority), true, can_certify)
                        }
                    }
                    Ok(None) => {
                        messages.add_message_from_string(
                            "The provided majority token doesn't exist".into(),
                            MessageKind::Error,
                        );
                        (None, false, false)
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
                        (None, false, false)
                    }
                }
            } else {
                (None, false, false)
            };
            Ok(Self {
                majority,
                have_access_to_major_only_content,
                can_certify,
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
