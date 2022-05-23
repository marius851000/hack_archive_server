use std::collections::BTreeSet;

use actix_web::{
    error::{ErrorNotFound, ErrorForbidden},
    get,
    web::{Data, Path},
    Result,
};
use maud::{html, PreEscaped};

use crate::{extractor::RequestData, AppData};

#[get("/index/hacks/{hack_id}")]
pub async fn index_hack(
    app_data: Data<AppData>,
    path: Path<String>,
    request_data: RequestData,
) -> Result<PreEscaped<String>> {
    let hack_id = path.into_inner();
    if let Some(hack) = app_data.storage.hacks.get(&hack_id) {
        let mut files = BTreeSet::new();
        files.insert("hack.json".to_string());
        for release in &hack.data.files {
            files.insert(release.filename.to_string());
        }
        for screenshot in &hack.data.screenshots {
            files.insert(screenshot.to_string());
        }
        if hack.need_majority_token(&app_data.storage.taginfo)
            && !request_data.have_access_to_major_only_content
        {
            return Err(ErrorForbidden(
                "A valid majority token is required to access this file",
            ));
        };
        Ok(html! {
            html {
                head {
                    meta charset="utf-8" {}
                    title { "index hack " (hack_id) }
                }
                body {
                    h1 { "Files for the " (hack.data.name) "." }
                    ul {
                        @for file in files {
                            li {
                                a href=(app_data.route_hack_file(&hack_id, &file)) { (file) }
                            }
                        }
                    }
                }
            }
        })
    } else {
        Err(ErrorNotFound("The hack doesn't exist"))
    }
}
