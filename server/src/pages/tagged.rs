use std::collections::HashMap;

use actix_web::{
    get,
    web::{Data, Path},
    HttpResponse,
};
use maud::{html, PreEscaped};
use pmd_hack_storage::{Query, Tag};

use crate::{extractor::RequestData, make_hack_list_hidden, wrap_page, AppData, PageInfo};

#[get("/tagged/{tag_id}")]
pub async fn tagged(
    app_data: Data<AppData>,
    path: Path<String>,
    request_data: RequestData,
) -> HttpResponse {
    let tag_id = path.into_inner();

    let base_query = Query::AtLeastOneOfTag(vec![Tag(tag_id.clone())]);

    let tag_info_single = app_data.storage.taginfo.get_tag(&Tag(tag_id));

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
            (make_hack_list_hidden(base_query, &request_data, &app_data))
        ),
        PageInfo {
            name: (request_data.lookup_with_args("hack-list-by-tag-title", &translation_args)),
            discourage_reload: false,
            display_majority_info: false, //TODO: enable if the tag is directly or indirectly major-only
        },
        &app_data,
        request_data,
    )
}
