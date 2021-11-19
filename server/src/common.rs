use crate::AppData;
use comrak::{ComrakOptions, markdown_to_html};
use maud::{Markup, PreEscaped, html};
use pmd_hack_storage::Hack;

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