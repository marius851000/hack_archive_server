use actix_web::{cookie::Cookie, get, http::StatusCode, web::Data, HttpResponse};
use maud::html;

use crate::AppData;

#[get("/index/hacks")]
pub async fn index_hacks(app_data: Data<AppData>) -> HttpResponse {
    let body = (html! {
        html {
            head {
                meta charset="utf-8" {}
                title { "index hacks" }
            }
            body {
                h1 { "Index of the hacks in the pmd hack archive" }
                ul {
                    @for hack_slug in app_data.storage.hacks.keys() {
                        li {
                            a href=(app_data.route_index_hack(hack_slug).as_str()) { (hack_slug) }
                        }
                    }
                }
            }
        }
    })
    .into_string();
    let mut response_builder = HttpResponse::build(StatusCode::OK);
    response_builder.content_type(mime::TEXT_HTML_UTF_8);
    response_builder.cookie(Cookie::build("messages", "").finish());

    response_builder.body(body.into_boxed_str().into_string())
}
