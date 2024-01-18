use std::{path::PathBuf, fs::File, io::Read};

use actix_files::NamedFile;
use actix_web::{Result, error::{ErrorNotFound, ErrorForbidden, ErrorBadRequest, ErrorInternalServerError}, Either};
use pmd_hack_storage::{Storage, Hack};
use zip::ZipArchive;

use crate::{AppData, extractor::RequestData};

pub type FileRefGetFileType = Either<NamedFile, Vec<u8>>;
use safe_join::SafeJoin;

pub enum FileRef {
    /// A file of an hack. First is hack id, second is the file name
    HackFile(String, String),
    /// A file inside a zip file of an hack. First is hack id, second if the zip file name, third is path inside zip
    Zipped(String, String, String)
}

impl FileRef {
    pub fn get_hack<'a>(&self, storage: &'a Storage) -> Option<&'a Hack> {
        let hack_id = match self {
            Self::HackFile(hack_id, _) => hack_id,
            Self::Zipped(hack_id, _, _) => hack_id
        };
        storage.hacks.get(hack_id)
    }

    pub fn get_file(&self, app_data: &AppData, request_data: &RequestData) -> Result<FileRefGetFileType> {
        let hack = if let Some(hack) = self.get_hack(&app_data.storage) {
            hack
        } else {
            return Err(ErrorNotFound(request_data.lookup("hack-does-not-exist")));
        };

        if hack.need_majority_token(&app_data.storage.taginfo) && !request_data.have_access_to_major_only_content {
            return Err(ErrorForbidden(
                request_data.lookup("valid-majority-token-needed-to-access-file"),
            ));
        }

        let filename = match self {
            Self::HackFile(_, filename) => filename,
            Self::Zipped(_, filename, _) => filename
        };

        let path: PathBuf = match hack.folder.safe_join(filename) {
            Ok(v) => v,
            Err(_) => {
                return Err(ErrorBadRequest(
                    request_data.lookup("path-traversal-detected"),
                ))
            }
        };

        Ok(match self {
            Self::HackFile(_, _) => Either::Left(NamedFile::open(path)?),
            Self::Zipped(_, _, inner_path) => {
                let file = File::open(path).map_err(ErrorNotFound)?;
                let mut zip = ZipArchive::new(file).map_err(ErrorInternalServerError)?;

                let mut file_to_return = zip.by_name(&inner_path).map_err(ErrorNotFound)?;
                let mut result = Vec::new();
                file_to_return.read_to_end(&mut result).map_err(ErrorInternalServerError)?;
                Either::Right(result)
            }
        })
    }
}