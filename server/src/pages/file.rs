use actix_web::{
    get,
    web::{Data, Path},
    Result,
};

use crate::{extractor::RequestData, AppData, FileRef, FileRefGetFileType};

#[get("/{hack_id}/{filename}")]
pub async fn file(
    app_data: Data<AppData>,
    path: Path<(String, String)>,
    request_data: RequestData,
) -> Result<FileRefGetFileType> {
    let storage = app_data.storage.load();
    let (hack_id, filename) = path.into_inner();
    let file_ref = FileRef::HackFile(hack_id, filename);

    return file_ref.get_file(&storage, &request_data);
}
