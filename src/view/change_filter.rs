use maud::{html, Markup};
use rocket::get;

use crate::{add_base, RequestData};

#[get("/change_filter")]
pub fn view_change_filter(data: RequestData) -> Markup {
    add_base(
        html! (
            p { "this page allow you to change the current filter used in this website" }
            p style="font-size:2em; color:red" { "CHANGING THIS VALUE MAY EXPOSE YOU TO OBSCENE CONTENT"}
            ul {
                @for (filter_id, filter) in &data.storage.filters.filters {
                    li {
                        (filter.label) b {" hides "}
                        @if filter.hide.len() == 0 {
                            "no tags "
                        } @else {
                            "the tags "
                            @for tag in &filter.hide {
                                b { (tag) } ", "
                            }
                        }
                        ". "
                        a href=(format!("/change_filter/{}", filter_id)) {
                            "use it"
                        }
                        "."
                    }
                }
            }
        ),
        "filter selection",
        &data,
    )
}
