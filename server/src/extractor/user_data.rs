use std::{convert::Infallible, future::Future, pin::Pin, sync::Arc};

use actix_web::{web::Data, FromRequest};
use database::{model::MajorityToken, HackClient};
use qstring::QString;

use crate::{
    message::{MessageKind, Messages},
    AppData,
};

pub struct UserData {
    pub majority: Option<MajorityToken>,
    pub have_access_to_major_only_content: bool,
    pub messages: Messages,
    pub majority_cookie_to_set: Option<String>,
}

impl FromRequest for UserData {
    type Error = Infallible;

    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let app_data = req.app_data::<Data<Arc<AppData>>>().unwrap().clone();
        if !app_data.use_majority_token {
            return Box::pin(async move {
                Ok(UserData {
                    majority: None,
                    have_access_to_major_only_content: false,
                    messages: Messages::default(),
                    majority_cookie_to_set: None,
                })
            });
        }

        //TODO: maybe put the majority token in post
        //TODO: button to remove the token
        let cookie = req.cookie("majority_token");
        let hackclient = req.app_data::<Data<HackClient>>().unwrap().clone();

        let query_string = QString::from(req.query_string());
        let parameter_majority_token = query_string
            .get("majority_token")
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
                match hackclient.get_majority_token(majority_token).await {
                    Ok(Some(majority)) => {
                        if majority.admin_flags.get().revoked {
                            messages.add_message_from_string(
                                "The provided majority token has been revoked by an administrator"
                                    .into(),
                                MessageKind::Error,
                            );
                            (Some(majority), false)
                        } else {
                            (Some(majority), true)
                        }
                    }
                    Ok(None) => {
                        messages.add_message_from_string(
                            "The provided majority token doesn't exist".into(),
                            MessageKind::Error,
                        );
                        (None, false)
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
