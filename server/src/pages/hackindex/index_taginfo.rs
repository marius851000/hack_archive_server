use actix_web::{error::ErrorInternalServerError, get, web::Data, Result};
use log::error;

use crate::AppData;

#[get("/index/taginfo.json")]
pub async fn index_taginfo(app_data: Data<AppData>) -> Result<String> {
    app_data.storage.taginfo.to_json().map_err(|e| {
        error!(
            "An error occured while generating the taginfo json file ! {:?}",
            e
        );
        ErrorInternalServerError("An error occured while generating the JSON file")
    })
}
