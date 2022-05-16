use actix_web::{get, web::Data};
use maud::{html, PreEscaped};

use crate::AppData;

#[get("/index/hacks")]
pub async fn index_hacks(app_data: Data<AppData>) -> PreEscaped<String> {
    html! {
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
                            a href=(app_data.route_index_hack(hack_slug)) { (hack_slug) }
                        }
                    }
                }
            }
        }
    }
}
