use actix_web::{
    get,
    web::{Data, Path},
    HttpResponse,
};
use maud::html;
use pmd_hack_storage::{Query, Tag};

use crate::{extractor::RequestData, make_hack_list, wrap_page, AppData, PageInfo};

#[get("/tagged/{tag_id}")]
pub async fn tagged(
    app_data: Data<AppData>,
    path: Path<String>,
    request_data: RequestData,
) -> HttpResponse {
    let tag_id = path.into_inner();

    let base_query = Query::AtLeastOneOfTag(vec![Tag(tag_id.clone())]);

    //TODO: share this code with the index page
    let unfiltered_hacks = (Query::Difference(
        Box::new(base_query),
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

    let tag_info_single = app_data.storage.taginfo.get_tag(&Tag(tag_id.clone()));

    // create the main page
    wrap_page(
        html!(
            h1 { "List of hacks with the tag " code { (tag_id) } }
            i { "Please note that this list may not be exaustive. Send me a message if an hack is missing in it." }
            @if let Some(tag_info_single) = tag_info_single {
                @if let Some(tag_description) = &tag_info_single.description {
                    p class="tagdescription" {
                        (tag_description)
                    }
                }
            }
            (make_hack_list(&unfiltered_hacks, &request_data, &app_data))
            @for (hidden_string, hidden_query) in &app_data.hidden_by_default {
                @let hidden_hacks = (Query::Intersection (
                    Box::new(hidden_query.clone()),
                    Box::new(Query::AtLeastOneOfTag(vec![Tag(tag_id.clone())]))
                )).get_matching(&app_data.storage).0;
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
            name: format!("Hack tagged {}", tag_id),
            discourage_reload: false,
        },
        &app_data,
        request_data,
    )
}
