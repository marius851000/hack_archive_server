use actix_web::{get, web::Data};
use maud::{html, PreEscaped};

use crate::AppData;

#[get("/index")]
pub async fn index_root(app_data: Data<AppData>) -> PreEscaped<String> {
    html! {
        html {
            head {
                meta charset="utf-8" {}
                title { "index root" }
            }
            body {
                h1 { "Index of the pmd hack archive" }
                p {
                    "Note: There may be hacks here that require a majority token to be mirrored. It should be put into the \"majority_token\" cookie. See "
                    a href=((app_data.route_simple_static(&["majority"]).as_str())) { "related page" }
                    " for more information. (it'll otherwise return an error when trying to access the hacks directory)."
                }
                ul {
                    li {
                        a href=(app_data.route_taginfo_file().as_str()) { "taginfo.json" }
                    }
                    li {
                        a href=(app_data.route_index_hacks().as_str()) { "hacks" }
                    }
                }
            }
        }
    }
}
