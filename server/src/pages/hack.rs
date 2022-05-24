use actix_web::{
    error::ErrorNotFound,
    get,
    web::{Data, Path},
    HttpResponse, Result,
};
use maud::html;

use crate::{
    extractor::RequestData, render_many_tags, render_markdown, render_tag, wrap_page, AppData,
    PageInfo,
};

#[get("/{hack_id}")]
pub async fn hack(
    app_data: Data<AppData>,
    path: Path<String>,
    request_data: RequestData,
) -> Result<HttpResponse> {
    let hack_id = path.into_inner();
    let hack = if let Some(hack) = app_data.storage.hacks.get(&hack_id) {
        hack
    } else {
        return Err(ErrorNotFound(
            "the given hack doesn't exist in the archive.",
        ));
    };

    let major_only_tags = hack.get_major_only_tags(&app_data.storage.taginfo);
    let major_only_hack = !major_only_tags.is_empty();
    if !major_only_hack || request_data.have_access_to_major_only_content {
        Ok(wrap_page(
            html!(
                h1 { (hack.data.name) }

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
                    (render_many_tags(all_tags.iter().cloned().collect(), &request_data, &app_data))
                }

                @if let Some(description) = &hack.data.description {
                    div class="hackdescription" {
                        (render_markdown(description))
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
                            img src=(app_data.route_hack_file(&hack_id, screenshot)) { }
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
                                    a href=(app_data.route_hack_file(&hack_id, &file.filename)) { "download" }
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
                                    (render_many_tags(file_tags.iter().cloned().collect(), &request_data, &app_data))
                                }
                            }
                        }
                    }
                }


            ),
            PageInfo {
                name: format!("Archive of {}", hack.data.name),
                discourage_reload: false,
                display_majority_info: major_only_hack,
            },
            &app_data,
            request_data,
        ))
    } else {
        Ok(wrap_page(
            html!(
                h1 { (format!("Major-only hack ({})", hack.data.name)) }
                p { "This hack is only available for major users. More information can be found on the "
                    a href=(app_data.route(&request_data, "majority")) { "dedicated page" } "."
                }
                p { "Reason of blocking :"}
                ul {
                    @for (tag_id, tag_info) in &major_only_tags {
                        li {
                            @if let Some(description) = tag_info.description.as_ref() {
                                (render_tag(tag_id, &request_data, &app_data)) " : " (description)
                            } @else {
                                "Undescripted tag " (render_tag(tag_id, &request_data, &app_data))
                            }
                        }
                    }
                }
            ),
            PageInfo {
                name: format!("Major-only hack {}", hack.data.name),
                discourage_reload: false,
                display_majority_info: true
            },
            &app_data,
            request_data,
        ))
    }
}
