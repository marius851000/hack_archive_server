use actix_web::{cookie::Cookie, post, web::Form, HttpResponse};
use serde::Deserialize;

use crate::{
    extractor::RequestData,
    message::{MessageKind, Messages},
    HttpResponseBuilderExtension,
};

#[derive(Deserialize)]
pub struct FormData {
    redirect_url: String,
}

#[post("/disconnect_majority_token")]
pub async fn disconnect_majority_token(
    form: Form<FormData>,
    request_data: RequestData,
) -> HttpResponse {
    HttpResponse::SeeOther()
        .cookie(Cookie::new("majority_token", ""))
        .append_header(("location", form.redirect_url.as_str()))
        .with_messages(Messages::create_with_message(
            request_data.lookup("message-majority-token-removed"),
            MessageKind::Success,
        ))
        .finish()
}
