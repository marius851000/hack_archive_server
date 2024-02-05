use super::{Hack, HackLoadError, TagInfoLoadError, Tags};
use crate::TagInfo;
use std::{
    collections::{HashMap, HashSet},
    fs::metadata,
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageLoadError {
    #[error("Can't list the sub-files of the folder {1:?}")]
    CantListFolder(#[source] io::Error, PathBuf),
    #[error("Can't get a specific entry when listing sub-files of the folder {1:?}")]
    CantGetSubfile(#[source] io::Error, PathBuf),
    #[error("Can't get the type of the file at {1:?}")]
    CantGetFileType(#[source] io::Error, PathBuf),
    #[error("Cant load the hack in the folder {1:?}")]
    CantLoadHack(#[source] HackLoadError, PathBuf),
    #[error("A warning is present for the the hack in the folder {1:?}")]
    NonFatalErrorLoadHack(#[source] HackLoadError, PathBuf),
    #[error("The file name \"{0}\" is an invalid UTF-8 string (sub-file of {1:?})")]
    InvalidFilename(String, PathBuf),
    #[error("Cant load the tag info file at {1:?}")]
    CantLoadTagInfo(#[source] TagInfoLoadError, PathBuf),
    #[error("What was supposed to be an hack folder at {0:?} is not a folder")]
    NotAFolderForHack(PathBuf),
    #[error("The file {1} for the hack {0} does not contain any tag in the category {2}")]
    TagForFileLacking(String, String, String),
    #[error("The tag {0} is used by {1:?} but doesn't exist in the taginfo file")]
    MissingTag(String, HashSet<String>),
}

impl StorageLoadError {
    pub fn is_not_much_important(&self) -> bool {
        match self {
            Self::MissingTag(_, _) => true,
            Self::TagForFileLacking(_, _, _) => true,
            _ => false,
        }
    }
}

#[derive(Default)]
pub struct Storage {
    pub hacks: HashMap<String, Hack>,
    pub tags: Tags,
    pub taginfo: TagInfo,
    pub errors: Vec<StorageLoadError>,
}

impl Storage {
    pub fn load_from_folder(root_folder: &Path) -> Self {
        let mut errors = Vec::new();
        let taginfo_path = root_folder.join("taginfo.json");
        let taginfo = match TagInfo::load_from_path(&taginfo_path)
            .map_err(|e| StorageLoadError::CantLoadTagInfo(e, taginfo_path.clone()))
        {
            Ok(v) => v,
            Err(err) => {
                errors.push(err);
                TagInfo::default()
            }
        };
        let mut result = Self {
            hacks: HashMap::new(),
            tags: Tags::default(),
            taginfo,
            errors,
        };
        let hacks_folder = root_folder.join("hacks");
        result.load_all_hacks_from_folder(&hacks_folder);
        result.warn_missing_tags();
        result
    }

    pub fn load_all_hacks_from_folder(&mut self, hacks_folder: &Path) {
        let dir_entry_iterator = match std::fs::read_dir(&hacks_folder) {
            Ok(v) => v,
            Err(e) => {
                self.errors.push(StorageLoadError::CantListFolder(
                    e,
                    hacks_folder.to_path_buf(),
                ));
                return;
            }
        };
        for hack_folder_maybe in dir_entry_iterator {
            let hack_folder = match hack_folder_maybe {
                Ok(v) => v,
                Err(e) => {
                    self.errors.push(StorageLoadError::CantGetSubfile(
                        e,
                        hacks_folder.to_path_buf(),
                    ));
                    continue;
                }
            };

            let hack_folder_path = hack_folder.path();
            let hack_folder_metadata = match metadata(&hack_folder_path) {
                Ok(v) => v,
                Err(e) => {
                    self.errors.push(StorageLoadError::CantGetFileType(
                        e,
                        hack_folder_path.clone(),
                    ));
                    continue;
                }
            };
            if !hack_folder_metadata.is_dir() {
                self.errors
                    .push(StorageLoadError::NotAFolderForHack(hack_folder_path));
                continue;
            };
            let hack_name = match hack_folder.file_name().to_str().map(|x| x.to_string()) {
                Some(v) => v,
                None => {
                    self.errors.push(StorageLoadError::InvalidFilename(
                        hack_folder.file_name().to_string_lossy().to_string(),
                        hacks_folder.to_path_buf(),
                    ));
                    continue;
                }
            };
            self.load_hack_from_folder(&hack_folder_path, &hack_name);
        }
    }

    fn load_hack_from_folder(&mut self, hack_folder_path: &Path, hack_name: &str) {
        let hack = match Hack::load_from_folder(hack_folder_path.to_path_buf(), &self.taginfo) {
            Ok((v, errors)) => {
                for error in errors {
                    self.errors.push(StorageLoadError::NonFatalErrorLoadHack(
                        error,
                        hack_folder_path.to_path_buf(),
                    ));
                }
                v
            }
            Err(e) => {
                self.errors.push(StorageLoadError::CantLoadHack(
                    e,
                    hack_folder_path.to_path_buf(),
                ));
                return;
            }
        };

        self.add_hack(hack_name.to_string(), hack);
    }

    fn add_hack(&mut self, name: String, hack: Hack) {
        for tag in hack.all_tags() {
            self.tags.add_hack_with_tag(&tag, name.clone());
        }
        // check that file have all required file tags
        for (category_tag, category_info) in &self.taginfo.categories {
            if category_info.required_for_file {
                for file in &hack.data.files {
                    let mut contain_appropriate_tag = false;
                    for tag in &file.tags {
                        if let Some(file_tag) = self.taginfo.get_tag(tag) {
                            if file_tag.category.as_ref() == Some(category_tag) {
                                contain_appropriate_tag = true;
                                break;
                            }
                        }
                    }
                    if !contain_appropriate_tag {
                        self.errors.push(StorageLoadError::TagForFileLacking(
                            name.to_string(),
                            file.filename.to_string(),
                            category_tag.to_string(),
                        ));
                    }
                }
            }
        }

        // actually insert the hack
        self.hacks.insert(name, hack);
    }

    fn warn_missing_tags(&mut self) {
        for (tag, users) in &self.tags.tag_list {
            if self.taginfo.get_tag(tag).is_none() {
                self.errors
                    .push(StorageLoadError::MissingTag(tag.to_string(), users.clone()));
            }
        }
    }
}
