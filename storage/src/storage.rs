use super::{Hack, HackLoadError, TagInfoLoadError, Tags};
use crate::TagInfo;
use log::warn;
use std::{
    collections::HashMap,
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
    #[error("The file name \"{0}\" is an invalid UTF-8 string (sub-file of {1:?})")]
    InvalidFilename(String, PathBuf),
    #[error("Cant load the tag info file at {1:?}")]
    CantLoadTagInfo(#[source] TagInfoLoadError, PathBuf),
}

#[derive(Default)]
pub struct Storage {
    pub hacks: HashMap<String, Hack>,
    pub tags: Tags,
    pub taginfo: TagInfo,
}

impl Storage {
    pub fn load_from_folder(root_folder: &Path) -> Result<Self, StorageLoadError> {
        let taginfo_path = root_folder.join("taginfo.json");
        let taginfo = TagInfo::load_from_path(&taginfo_path)
            .map_err(|e| StorageLoadError::CantLoadTagInfo(e, taginfo_path.clone()))?;
        let mut result = Self {
            hacks: HashMap::new(),
            tags: Tags::default(),
            taginfo,
        };
        let hacks_folder = root_folder.join("hacks");
        result.load_all_hacks_from_folder(&hacks_folder)?;
        Ok(result)
    }

    pub fn load_all_hacks_from_folder(
        &mut self,
        hacks_folder: &Path,
    ) -> Result<(), StorageLoadError> {
        for hack_folder_maybe in std::fs::read_dir(&hacks_folder)
            .map_err(|e| StorageLoadError::CantListFolder(e, hacks_folder.to_path_buf()))?
        {
            let hack_folder = hack_folder_maybe
                .map_err(|e| StorageLoadError::CantGetSubfile(e, hacks_folder.to_path_buf()))?;
            let hack_folder_path = hack_folder.path();
            let hack_folder_metadata = metadata(&hack_folder_path)
                .map_err(|e| StorageLoadError::CantGetFileType(e, hack_folder_path.clone()))?;
            if !hack_folder_metadata.is_dir() {
                println!(
                    "warning: {:?} isn't a directory (in the hack list folder)",
                    hack_folder_path
                );
                continue;
            };
            let hack_name = match hack_folder.file_name().to_str().map(|x| x.to_string()) {
                Some(v) => v,
                None => {
                    return Err(StorageLoadError::InvalidFilename(
                        hack_folder.file_name().to_string_lossy().to_string(),
                        hacks_folder.to_path_buf(),
                    ))
                }
            };
            self.load_hack_from_folder(&hack_folder_path, &hack_name)?;
        }
        Ok(())
    }

    fn load_hack_from_folder(
        &mut self,
        hack_folder_path: &Path,
        hack_name: &str,
    ) -> Result<(), StorageLoadError> {
        self.add_hack(
            hack_name.to_string(),
            Hack::load_from_folder(hack_folder_path.to_path_buf(), &self.taginfo)
                .map_err(|e| StorageLoadError::CantLoadHack(e, hack_folder_path.to_path_buf()))?,
        )?;
        Ok(())
    }

    fn add_hack(&mut self, name: String, hack: Hack) -> Result<(), StorageLoadError> {
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
                        log::warn!("The file {} for the hack {} doesn't contain a tag with the category {}, as it is required by the category", file.filename, name, category_tag);
                    }
                }
            }
        }

        // actually insert the hack
        self.hacks.insert(name, hack);
        Ok(())
    }

    pub fn warn_missing_tags(&self) {
        for (tag, users) in &self.tags.tag_list {
            if self.taginfo.get_tag(tag).is_none() {
                warn!(
                    "The tag {} is used by {:?} but doesn't exist in the taginfo file",
                    tag, users
                );
            }
        }
    }
}
