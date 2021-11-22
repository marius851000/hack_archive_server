mod index;
pub use index::index_page;

mod hack;
pub use hack::hack_page;

mod file;
pub use file::file_page;

mod css;
pub use css::css_page;

mod common;
pub use common::*;
use pmd_hack_storage::{Query, Storage};

use actix_web::get;
#[get("/Oswald-Medium.ttf")]
pub async fn oswald() -> &'static [u8] {
    include_bytes!("../Oswald-Medium.ttf")
}

pub struct AppData {
    pub root_url: String,
    pub storage: Storage,
    /// String: description of the reason
    /// Query: when does it match
    pub hidden_by_default: Vec<(String, Query)>,
}
