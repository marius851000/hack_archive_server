use crate::AppData;
use comrak::{markdown_to_html, ComrakOptions};
use maud::{html, Markup, PreEscaped};
use pmd_hack_storage::{Hack, Tag};

pub struct PageInfo {
    pub name: String,
}

pub fn wrap_page(markup: Markup, page_info: PageInfo, app_data: &AppData) -> Markup {
    html!(
        html {
            head {
                title { (page_info.name) }
                link rel="stylesheet" href=(format!("{}/style.css", app_data.root_url)) {}
            }
            body {
                header {
                    a id="archivedlink" href=(app_data.root_url) { "Archived hacks list" }
                    a id="newslink" href="https://hacknews.pmdcollab.org/" { "Return to the news site" }
                }
                main {
                    (markup)
                }
                footer {
                    p {
                        "Archive created and maintained by marius851000 ("
                        code { "marius851000#2522" }
                        " on Discord). This site is not directly affiliated, and not to be confused with the "
                        a href="https://hacks.skytemple.org" { span class="skytemple" {"SkyTemple"} " hack list" } "."
                    }
                    p {
                        "Site data can be mirrored with rclone using the http directory at "
                        a href="https://hacknews.pmdcollab.org/archive" { "hacknews.pmdcollab.org/archive" }
                        "."
                    }
                    p {
                        "Source code of the site avalaible on " a href="https://github.com/marius851000/hack_archive_server" { "GitHub" } "."
                    }
                }
            }
        }
    )
}

pub fn make_hack_list(hacks: &[(String, &Hack)], app_data: &AppData) -> Markup {
    html! {
        ul {
            @for (hack_id, hack) in hacks {
                li {
                    a href=(format!("{}/{}", app_data.root_url, hack_id)) {
                        (hack.data.name)
                    }
                }
            }
        }
    }
}

pub fn render_markdown(text: &str) -> PreEscaped<String> {
    PreEscaped(markdown_to_html(text, &ComrakOptions::default()))
}

pub fn render_tag(tag: &Tag, app_data: &AppData) -> Markup {
    html! {
        a href=(format!("{}/tagged/{}", app_data.root_url, tag.0)) {
            @if let Some(single_tag_info) = app_data.storage.taginfo.get_tag(tag) {
                @let label = single_tag_info.label.as_ref().unwrap_or(&tag.0);
                @if let Some(category_data) = app_data.storage.taginfo.get_category_for_single_tag_info(single_tag_info, tag) {
                    span class="tag" style=(format!("border-color:{};background-color:{}", category_data.border_color, category_data.background_color)) { (label) }
                } @else {
                    span class="tag" { (label) }
                }
            } @else {
                span class="tag" { (tag.0) }
            }
        }
    }
}

pub fn render_many_tags(tags: Vec<Tag>, app_data: &AppData) -> Markup {
    let tags = app_data.storage.taginfo.orders_tags(tags);
    html! {
        p class="tagslist" {
            "tags : "
            @for (count, tag) in tags.iter().enumerate() {
                @let remaining = tags.len() - count - 1;
                (render_tag(tag, app_data));
                @match remaining {
                    1 => ", and",
                    2.. => ", ",
                    _ => ""
                }
            }
        }
    }
}
