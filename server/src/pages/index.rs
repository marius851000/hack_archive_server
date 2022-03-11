use std::sync::Arc;

use actix_web::{get, web::Data, HttpResponse};
use maud::html;
use pmd_hack_storage::Query;

use crate::{extractor::UserData, make_hack_list, wrap_page, AppData, PageInfo};

#[get("/")]
pub async fn index(app_data: Data<Arc<AppData>>, user_data: UserData) -> HttpResponse {
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
            h1 { "Marius's archive of PMD hack-rom" }
            p {
                "This is the part of my archive that store rom-hacks patches. "
                "The goal of this archive is to save every version of every hacks. "
            }
            p {
                "If you see there is an hack or a version that is missing, don't hesitate to contact me on Discord at marius851000#2522 (or any other one)."
            }
            h2 { "List of hacks" }
            (make_hack_list(&unfiltered_hacks, &app_data))
            @for (hidden_string, hidden_query) in &app_data.hidden_by_default {
                @let hidden_hacks = hidden_query.get_matching(&app_data.storage).0;
                @if !hidden_hacks.is_empty() {
                    details {
                        summary {
                            (hidden_string) " (click to reveal)"
                        }
                        (make_hack_list(&hidden_hacks, &app_data))
                    }
                }
            }
        ),
        PageInfo {
            name: "Archive of PMD hacks".into(),
        },
        &app_data,
        user_data,
    )
}
