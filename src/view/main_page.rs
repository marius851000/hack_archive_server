use maud::{html, Markup};
use rocket::get;

use crate::{add_base, RequestData};

#[get("/")]
pub fn view_main_page(data: RequestData) -> Markup {
    add_base(
        html! {
            h1 { "main page" }
        },
        "main page",
        &data,
    )
}
