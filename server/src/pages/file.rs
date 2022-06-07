use actix_files::NamedFile;
use actix_web::{
    error::{ErrorBadRequest, ErrorForbidden, ErrorNotFound},
    get,
    web::{Data, Path},
    Result,
};
use safe_join::SafeJoin;

use crate::{extractor::RequestData, AppData};

#[get("/{hack_id}/{filename}")]
pub async fn file(
    app_data: Data<AppData>,
    path: Path<(String, String)>,
    request_data: RequestData,
) -> Result<NamedFile> {
    let (hack_id, filename) = path.into_inner();
    let hack = if let Some(hack) = app_data.storage.hacks.get(&hack_id) {
        hack
    } else {
        return Err(ErrorNotFound(request_data.lookup("hack-does-not-exist")));
    };
    if hack.need_majority_token(&app_data.storage.taginfo)
        && !request_data.have_access_to_major_only_content
    {
        return Err(ErrorForbidden(
            request_data.lookup("valid-majority-token-needed-to-access-file"),
        ));
    };
    let path = match hack.folder.safe_join(filename) {
        Ok(v) => v,
        Err(_) => {
            return Err(ErrorBadRequest(
                request_data.lookup("path-traversal-detected"),
            ))
        }
    };
    Ok(NamedFile::open(path)?)
}
