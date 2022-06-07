use actix_web::{
    cookie::Cookie,
    post,
    web::{Data, Form},
    HttpResponse,
};
use serde::Deserialize;

use crate::{
    extractor::RequestData,
    message::{MessageKind, Messages},
    AppData, HttpResponseBuilderExtension,
};

#[derive(Deserialize)]
pub struct FormData {
    redirect_url: String,
    majority_token: String,
}

#[post("/connect_majority_token")]
pub async fn connect_majority_token(
    form: Form<FormData>,
    request_data: RequestData,
    app_data: Data<AppData>,
) -> HttpResponse {
    let mut messages = Messages::default();
    let mut builder = HttpResponse::SeeOther();
    if let (Some(_), true, _) = app_data
        .check_validity_of_majority_token(
            &form.majority_token,
            &mut messages,
            &request_data.language,
        )
        .await
    {
        messages.add_message_from_string(
            request_data.lookup("message-majority-token-added"),
            MessageKind::Success,
        );
        builder.cookie(Cookie::new(
            "majority_token",
            form.majority_token.to_string(),
        ));
    };
    builder
        .append_header(("location", form.redirect_url.as_str()))
        .with_messages(messages)
        .finish()
}
