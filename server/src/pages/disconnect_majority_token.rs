use actix_web::{
    cookie::Cookie,
    post,
    web::{Data, Form},
    HttpResponse,
};
use serde::Deserialize;

use crate::AppData;

#[derive(Deserialize)]
pub struct FormData {
    redirect_url: String,
}

#[post("/disconnect_majority_token")]
pub async fn disconnect_majority_token(
    form: Form<FormData>,
    app_data: Data<AppData>,
) -> HttpResponse {
    HttpResponse::SeeOther()
        .cookie(Cookie::new("majority_token", ""))
        .append_header((
            "location",
            app_data
                .add_get_param_or_root_with_redirect_error(
                    &form.redirect_url,
                    "disconnect_success",
                    "true",
                )
                .as_str(),
        ))
        .finish()
}
