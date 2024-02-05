//! This modules should contains every page generation content

pub mod majority;

pub mod connect_majority_token;
pub mod create_majority_token;
pub mod css;
pub mod decompress;
pub mod disconnect_majority_token;
pub mod file;
pub mod hack;
pub mod hackindex;
pub mod index;
pub mod reload_storage;
pub mod tagged;

use actix_web::get;

#[get("/Oswald-Medium.ttf")]
pub async fn oswald() -> &'static [u8] {
    include_bytes!("../../Oswald-Medium.ttf")
}
