use std::collections::HashMap;

use actix_web::{
    get,
    web::{Data, Path},
    HttpResponse,
};
use maud::{html, PreEscaped};
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
    let mut translation_args = HashMap::new();
    translation_args.insert("tag", tag_id.clone().into());
    wrap_page(
        html!(
            h1 { (PreEscaped(request_data.lookup_with_args("hack-list-by-tag-header", &translation_args))) }
            i { (request_data.lookup("hack-list-by-tag-non-exaustive-note")) }
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
                            (hidden_string) " " (request_data.lookup("click-to-reveal-button"))
                        }
                        (make_hack_list(&hidden_hacks, &request_data, &app_data))
                    }
                }
            }
        ),
        PageInfo {
            name: (request_data.lookup_with_args("hack-list-by-tag-title", &translation_args)),
            discourage_reload: false,
        },
        &app_data,
        request_data,
    )
}
