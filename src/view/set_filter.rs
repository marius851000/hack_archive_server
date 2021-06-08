use rocket::{get, response::Redirect, uri};

use crate::RequestData;

#[get("/change_filter/<filter_slug>")]
pub fn view_set_filter(mut data: RequestData, filter_slug: String) -> Option<Redirect> {
    if !data.set_filter(&filter_slug) {
        return None;
    }

    Some(Redirect::to(uri!(super::view_main_page)))
}
