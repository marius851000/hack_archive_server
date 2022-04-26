use actix_web::{get, web::Data, HttpResponse};
use maud::{html, PreEscaped};
use pmd_hack_storage::Query;

use crate::{extractor::RequestData, make_hack_list, wrap_page, AppData, PageInfo};

#[get("/")]
pub async fn index(app_data: Data<AppData>, request_data: RequestData) -> HttpResponse {
    let unfiltered_hacks = (Query::Difference(
        Box::new(Query::All),
        Box::new(Query::Or(
            app_data
                .hidden_by_default
                .iter()
                .map(|(_t, q)| q.clone())
                .collect(),
        )),
    ))
    .get_matching(&app_data.storage)
    .0;

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
            h2 { "List of hacks" }
            (make_hack_list(&unfiltered_hacks, &request_data, &app_data))
            @for (hidden_string, hidden_query) in &app_data.hidden_by_default {
                @let hidden_hacks = hidden_query.get_matching(&app_data.storage).0;
                @if !hidden_hacks.is_empty() {
                    details {
                        summary {
                            (hidden_string) " (click to reveal)"
                        }
                        (make_hack_list(&hidden_hacks, &request_data, &app_data))
                    }
                }
            }
        ),
        PageInfo {
            name: "Archive of PMD hacks".into(),
            discourage_reload: false,
        },
        &app_data,
        request_data,
    )
}
