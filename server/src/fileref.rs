use std::{
    io::{Cursor, Read, Seek},
    path::PathBuf,
};

use actix_files::NamedFile;
use actix_web::{
    error::{ErrorBadRequest, ErrorForbidden, ErrorInternalServerError, ErrorNotFound},
    Either, Result,
};
use pmd_hack_storage::{Hack, Storage};
use zip::ZipArchive;

use crate::extractor::RequestData;

pub type FileRefGetFileType = Either<NamedFile, Vec<u8>>;
use safe_join::SafeJoin;

pub trait ReadSeek: std::io::Read + Seek {}
impl ReadSeek for Cursor<Vec<u8>> {}

pub enum FileRef {
    /// A file of an hack. First is hack id, second is the file name
    HackFile(String, String),
    /// A file inside a zip file of an hack. First is hack id, second if the zip file name, third is path inside zip
    Zipped(Box<FileRef>, String),
}

impl FileRef {
    pub fn get_hack<'a>(&self, storage: &'a Storage) -> Option<&'a Hack> {
        let hack_id = match self {
            Self::HackFile(hack_id, _) => hack_id,
            Self::Zipped(hack_source, _) => return hack_source.get_hack(storage),
        };
        storage.hacks.get(hack_id)
    }

    pub fn get_file(
        &self,
        storage: &Storage,
        request_data: &RequestData,
    ) -> Result<FileRefGetFileType> {
        let hack = if let Some(hack) = self.get_hack(&storage) {
            hack
        } else {
            return Err(ErrorNotFound(request_data.lookup("hack-does-not-exist")));
        };

        if hack.need_majority_token(&storage.taginfo)
            && !request_data.have_access_to_major_only_content
        {
            return Err(ErrorForbidden(
                request_data.lookup("valid-majority-token-needed-to-access-file"),
            ));
        }

        let filename = match self {
            Self::HackFile(_, filename) => filename,
            Self::Zipped(_, filename) => filename,
        };

        Ok(match self {
            Self::HackFile(_, _) => {
                let path: PathBuf = match hack.folder.safe_join(filename) {
                    Ok(v) => v,
                    Err(_) => {
                        return Err(ErrorBadRequest(
                            request_data.lookup("path-traversal-detected"),
                        ))
                    }
                };

                Either::Left(NamedFile::open(path)?)
            }
            Self::Zipped(source_hack, inner_path) => {
                let hack_file = source_hack.get_reader(&storage, &request_data)?;

                //ErrorNotFound
                let mut zip = ZipArchive::new(hack_file).map_err(ErrorInternalServerError)?;

                let mut file_to_return = zip.by_name(&inner_path).map_err(ErrorNotFound)?;
                let mut result = Vec::new();
                file_to_return
                    .read_to_end(&mut result)
                    .map_err(ErrorInternalServerError)?;
                Either::Right(result)
            }
        })
    }

    pub fn get_reader(
        &self,
        storage: &Storage,
        request_data: &RequestData,
    ) -> Result<Box<dyn ReadSeek>> {
        Ok(match self.get_file(&storage, &request_data)? {
            Either::Left(mut named_file) => {
                let mut file_content = Vec::new();
                named_file
                    .read_to_end(&mut file_content)
                    .map_err(|_| ErrorBadRequest(request_data.lookup("message-error-file-open")))?;

                Box::new(Cursor::new(file_content))
            }
            Either::Right(byte_vec) => Box::new(Cursor::new(byte_vec)),
        })
    }
}
