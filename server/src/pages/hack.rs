use std::sync::Arc;

use actix_web::{
    error::ErrorNotFound,
    get,
    web::{Data, Path},
    HttpResponse, Result,
};
use maud::html;

use crate::{extractor::UserData, render_many_tags, render_markdown, wrap_page, AppData, PageInfo};

#[get("/{hack_id}")]
pub async fn hack(
    app_data: Data<Arc<AppData>>,
    path: Path<String>,
    user_data: UserData,
) -> Result<HttpResponse> {
    let hack_id = path.into_inner();
    let hack = if let Some(hack) = app_data.storage.hacks.get(&hack_id) {
        hack
    } else {
        return Err(ErrorNotFound(
            "the given hack doesn't exist in the archive.",
        ));
    };

    Ok(wrap_page(
        html!(
            h1 { (hack.data.name) }

            @if let Some(description) = &hack.data.description {
                div class="hackdescription" {
                    (render_markdown(description))
                }
            }

            @if !hack.data.authors.is_empty() {
                p id="authorlist" {
                    "made by : "
                    @for (remaining, author) in hack.data.authors.iter().rev().enumerate().rev() {
                        span class="person" { (author) }
                        @match remaining {
                            1 => " and ",
                            2.. => ", ",
                            _ => "",
                        }
                    }
                }
            }

            @let all_tags = hack.all_tags();

            @if !all_tags.is_empty() {
                (render_many_tags(all_tags.iter().cloned().collect(), &app_data))
            }

            @if let Some(source) = &hack.data.source {
                p { "source : " a href=(source) { (source) }}
            }

            @if let Some(skytemple_db_id) = &hack.data.skytemple_db_id {
                p {
                    a href=(format!("https://hacks.skytemple.org/h/{}", skytemple_db_id)) {
                        "See on the " span class="skytemple" { "SkyTemple" } " hack list"
                    }
                    " (under the id " code { (skytemple_db_id) } ")."
                }
            }

            @if !hack.data.screenshots.is_empty() {
                p { "screenshots" }
                div class="screenshots" {
                    @for screenshot in &hack.data.screenshots {
                        img src=(format!("{}/{}/{}", app_data.root_url, hack_id, screenshot)) { }
                    }
                }
            }

            @if !hack.data.links.is_empty() {
                p { "external links" }
                ul {
                    @for (name, url) in &hack.data.links {
                        li {
                            a href=(url) { (name) }
                        }
                    }
                }
            }

            h2 { "files" }
            @if hack.data.files.is_empty() {
                p { "no file" }
            } else {
                div class="filelist" {
                    @for file in &hack.data.files {
                        div class="hack" {
                            h4 { (file.label) }
                            p {
                                a href=(format!("{}/{}/{}", app_data.root_url, hack_id, file.filename)) { "download" }
                            }
                            /*@if let Some(description) = &file.description {
                                @let rendered = render_markdown(description);
                                @if description.len() < 500 && description.matches('\n').count() < 6 {
                                    div class="filedescription" { (rendered) }
                                }
                                @else {
                                    details {
                                        summary {
                                            @let rendered_preview = render_markdown(&description.split('\n').next().unwrap().chars().take(60).collect::<String>());
                                            (PreEscaped(rendered_preview.0.replace("<p>", "").replace("</p>", "")))
                                            "..."
                                        }
                                        (rendered)
                                    }
                                }

                            }*/
                            //TODO: find a good way to present this
                            @let file_tags = &file.get_all_tags();
                            @if !file_tags.is_empty() {
                                (render_many_tags(file_tags.iter().cloned().collect(), &app_data))
                            }
                        }
                    }
                }
            }


        ),
        PageInfo {
            name: format!("Archive of {}", hack.data.name),
            discourage_reload: false,
        },
        &app_data,
        user_data,
    ))
}
