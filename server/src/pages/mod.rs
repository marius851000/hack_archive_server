//! This modules should contains every page generation content

pub mod majority;

pub mod create_majority_token;
pub mod css;
pub mod disconnect_majority_token;
pub mod file;
pub mod hack;
pub mod index;
pub mod tagged;

use actix_web::get;

#[get("/Oswald-Medium.ttf")]
pub async fn oswald() -> &'static [u8] {
    include_bytes!("../../Oswald-Medium.ttf")
}
