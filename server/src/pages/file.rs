use actix_web::{
    get,
    web::{Data, Path},
    Result,
};

use crate::{extractor::RequestData, AppData, FileRef, FILE_REF_GET_FILE_TYPE};

#[get("/{hack_id}/{filename}")]
pub async fn file(
    app_data: Data<AppData>,
    path: Path<(String, String)>,
    request_data: RequestData,
) -> Result<FILE_REF_GET_FILE_TYPE> {
    let (hack_id, filename) = path.into_inner();
    let file_ref = FileRef::HackFile(hack_id, filename);
    
    return file_ref.get_file(&app_data, &request_data);
}
