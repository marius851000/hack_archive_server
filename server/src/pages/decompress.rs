use actix_web::{
    error::ErrorInternalServerError,
    get,
    web::{Data, Path},
    Either, HttpResponse, Result,
};
use maud::html;
use zip::ZipArchive;

use crate::{extractor::RequestData, wrap_page, AppData, FileRef, FileRefGetFileType};

#[get("/decompress/{hack_id}/{filename}/{tail:.*}")]
pub async fn decompress(
    app_data: Data<AppData>,
    path: Path<(String, String, String)>,
    request_data: RequestData,
) -> Result<Either<HttpResponse, FileRefGetFileType>> {
    let storage = app_data.storage.load();
    let (hack_id, filename, inner_path) = path.into_inner();
    let file_ref = FileRef::HackFile(hack_id.clone(), filename.clone());

    if inner_path.is_empty() {
        let file = file_ref.get_reader(&storage, &request_data)?;
        let mut zip = ZipArchive::new(file).map_err(ErrorInternalServerError)?;

        let mut list_of_files = Vec::with_capacity(zip.len());
        for file_id in 0..zip.len() {
            let file = zip.by_index(file_id).map_err(ErrorInternalServerError)?;
            if file.is_file() {
                let name = file.name().to_string();
                list_of_files.push(name);
            }
        }

        list_of_files.sort();

        return Ok(Either::Left(wrap_page(
            html! {
                ul {
                    h1 { (format!("List of files in {} of {}", filename, file_ref.get_hack(&storage).unwrap().data.name))}
                    @for file_path in &list_of_files {
                        li {
                            a href=(app_data.route_hack_decompress_file(&hack_id, &filename, file_path).as_str()) { (file_path) }
                        }
                    }
                }
            },
            crate::PageInfo {
                name: format!("browsing {}", filename),
                discourage_reload: false,
                display_majority_info: false,
            },
            &app_data,
            request_data,
        )));
    } else {
        let sub_file = FileRef::Zipped(Box::new(file_ref), inner_path);
        Ok(Either::Right(sub_file.get_file(&storage, &request_data)?))
    }
}
