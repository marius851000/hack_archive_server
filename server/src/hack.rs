use std::sync::Arc;

use actix_web::{
    error::ErrorNotFound,
    get,
    web::{Data, Path},
    Result,
};
use maud::{html, Markup};

use crate::{wrap_page, AppData, PageInfo, render_markdown};

#[get("/{hack_id}")]
pub async fn hack_page(
    app_data: Data<Arc<AppData>>,
    Path(hack_id): Path<String>,
) -> Result<Markup> {
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
                        @if remaining > 1 {
                            ", "
                        } @else if remaining == 1 {
                            " and "
                        }
                    }
                }
            }

            @if !hack.data.tags.is_empty() {
                p id="tagslist" {
                    "tags : "
                    @for (count, tag) in hack.data.tags.iter().enumerate() {
                        @let remaining = hack.data.tags.len() - count - 1;
                        span class="tag" { (tag) }
                        @if remaining > 1 {
                            ", "
                        } @else if remaining == 1 {
                            " and "
                        }
                    }
                }
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
                            @if let Some(base) = &file.base {
                                //TODO: human-compatible name (create a new json file to store this ?)
                                p { (format!("base rom : {}", base)) }
                            }
                            @if !file.language.is_empty() {
                                p {
                                    @if file.language.len() == 1 {
                                        "language"
                                    } @else {
                                        "languages"
                                    }
                                    " : "
                                    @for (remaining, language) in file.language.iter().rev().enumerate().rev() {
                                        span class="language" { (language) }
                                        @if remaining == 1 {
                                            " and "
                                        } @else if remaining > 1 {
                                            ", "
                                        }
                                    }
                            }
                            }
                        }
                    }
                }
            }


        ),
        PageInfo {
            name: format!("Archive of {}", hack.data.name),
        },
        &app_data
    ))
}
