use maud::{Markup, html, DOCTYPE};

use crate::RequestData;

pub struct SiteData {
    pub name: String
}

pub fn add_base(body: Markup, tab_title: &str, data: &RequestData) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                title { (data.site_data.name) "-" (tab_title) }
            }
            body {
                header {
                    "current filter : " (data.get_current_filter().label)
                }
                "You have the header here..."
                (body)
            }
        }
    }
}