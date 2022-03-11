use std::sync::Arc;

use actix_files::NamedFile;
use actix_web::{
    error::{ErrorBadRequest, ErrorNotFound},
    get,
    web::{Data, Path},
    Result,
};
use safe_join::SafeJoin;

use crate::AppData;

#[get("/{hack_id}/{filename}")]
pub async fn file(
    app_data: Data<Arc<AppData>>,
    Path((hack_id, filename)): Path<(String, String)>,
) -> Result<NamedFile> {
    let hack = if let Some(hack) = app_data.storage.hacks.get(&hack_id) {
        hack
    } else {
        return Err(ErrorNotFound("The given hack doesn't exist"));
    };
    let path = match hack.folder.safe_join(filename) {
        Ok(v) => v,
        Err(_) => return Err(ErrorBadRequest("A path traversal attack was detected")),
    };
    Ok(NamedFile::open(path)?)
}
