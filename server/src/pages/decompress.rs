use std::{fs::File, io::Read};
use actix_web::{web::{Data, Path}, get, HttpResponse, Result, error::{ErrorBadRequest, ErrorForbidden, ErrorNotFound, ErrorInternalServerError}, Either};
use maud::html;
use safe_join::SafeJoin;
use zip::ZipArchive;

use crate::{AppData, extractor::RequestData, wrap_page};

#[get("/decompress/{hack_id}/{filename}/{tail:.*}")]
pub async fn decompress(
    app_data: Data<AppData>,
    path: Path<(String, String, String)>,
    request_data: RequestData,
) -> Result<Either<HttpResponse, Vec<u8>>> {
    //TODO: also, return 404 when a path is not found
    //TODO: this section is already used in file, do not duplicate it!
    let (hack_id, filename, inner_path) = path.into_inner();
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
    let path = match hack.folder.safe_join(&filename) {
        Ok(v) => v,
        Err(_) => {
            return Err(ErrorBadRequest(
                request_data.lookup("path-traversal-detected"),
            ))
        }
    };
    // end of todo section.

    let file = File::open(path).map_err(ErrorNotFound)?;
    let mut zip = ZipArchive::new(file).map_err(ErrorInternalServerError)?;

    if inner_path.is_empty() {
        let mut list_of_files = Vec::with_capacity(zip.len());
        for file_id in 0..zip.len() {
            let file = zip.by_index(file_id).map_err(ErrorInternalServerError)?;
            if file.is_file() {
                let name = file.name().to_string();
                list_of_files.push(name);
            }
        }

        list_of_files.sort();

        return Ok(Either::Left(wrap_page(html!{
            ul {
                h1 { (format!("List of files in {} of {}", filename, hack.data.name))}
                @for file_path in &list_of_files {
                    li {
                        a href=(app_data.route_hack_decompress_file(&hack_id, &filename, file_path).as_str()) { (file_path) }
                    }
                }
            }
        }, crate::PageInfo { name: format!("browsing {}", filename), discourage_reload: false, display_majority_info: false }, &app_data, request_data)));
    } else {
        let mut file_to_return = zip.by_name(&inner_path).map_err(ErrorNotFound)?;
        let mut result = Vec::new();
        file_to_return.read_to_end(&mut result).map_err(ErrorInternalServerError)?;
        return Ok(Either::Right(result))
    }
}