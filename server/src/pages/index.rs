use actix_web::{get, web::Data, HttpResponse};
use maud::{html, PreEscaped};
use pmd_hack_storage::Query;

use crate::{extractor::RequestData, make_hack_list_hidden, wrap_page, AppData, PageInfo};

#[get("/")]
pub async fn index(app_data: Data<AppData>, request_data: RequestData) -> HttpResponse {
    // create the main page
    wrap_page(
        html!(
            h1 { (request_data.lookup("website-title")) }
            p {
                (request_data.lookup("landpage-presentation"))
            }
            p {
                (PreEscaped(request_data.lookup("landpage-missing")))
            }
            h2 { (request_data.lookup("landpage-list-of-hacks")) }
            (make_hack_list_hidden(Query::All, &request_data, &app_data))
        ),
        PageInfo {
            name: request_data.lookup("landpage-title"),
            discourage_reload: false,
            display_majority_info: false,
        },
        &app_data,
        request_data,
    )
}
