use actix_web::{cookie::Cookie, post, web::Form, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FormData {
    redirect_url: String,
}

#[post("/disconnect_majority_token")]
pub async fn disconnect_majority_token(form: Form<FormData>) -> HttpResponse {
    HttpResponse::SeeOther()
        .cookie(Cookie::new("majority_token", ""))
        .append_header(("location", form.redirect_url.to_string()))
        .finish()
}
