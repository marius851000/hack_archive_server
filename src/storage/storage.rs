use super::{Filters, FiltersLoadError, Hack, HackLoadError, Tags};
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
    #[error("The file at {0:?} isn't a directory, but is child of the hack storage folder {1:?}")]
    NotADirectory(PathBuf, PathBuf),
    #[error("The file name \"{0}\" is an invalid UTF-8 string (sub-file of {:?})")]
    InvalidFilename(String, PathBuf),
    #[error("Can't load filters")]
    CantLoadFilters(#[from] FiltersLoadError),
}

#[derive(Default)]
pub struct Storage {
    pub hacks: HashMap<String, Hack>,
    pub filters: Filters,
    pub tags: Tags,
}

impl Storage {
    pub fn load_from_folder(root_folder: &Path) -> Result<Self, StorageLoadError> {
        let hacks_folder = root_folder.join("hacks");
        let filter_path = root_folder.join("filters.json");
        let mut result = Self::default();

        result.load_all_hacks_from_folder(&hacks_folder)?;
        result.load_filters(&filter_path)?;
        Ok(result)
    }

    fn load_all_hacks_from_folder(&mut self, hacks_folder: &Path) -> Result<(), StorageLoadError> {
        for hack_folder_maybe in std::fs::read_dir(&hacks_folder)
            .map_err(|e| StorageLoadError::CantListFolder(e, hacks_folder.to_path_buf()))?
        {
            let hack_folder = hack_folder_maybe
                .map_err(|e| StorageLoadError::CantGetSubfile(e, hacks_folder.to_path_buf()))?;
            let hack_folder_path = hack_folder.path();
            let hack_folder_metadata = metadata(&hack_folder_path)
                .map_err(|e| StorageLoadError::CantGetFileType(e, hack_folder_path.clone()))?;
            if !hack_folder_metadata.is_dir() {
                return Err(StorageLoadError::NotADirectory(
                    hack_folder_path.to_path_buf(),
                    hacks_folder.to_path_buf(),
                ));
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
            Hack::load_from_folder(hack_folder_path.to_path_buf())
                .map_err(|e| StorageLoadError::CantLoadHack(e, hack_folder_path.to_path_buf()))?,
        )?;
        Ok(())
    }

    fn add_hack(&mut self, name: String, hack: Hack) -> Result<(), StorageLoadError> {
        for tag in hack.data.tags.iter() {
            self.tags.add_hack_with_tag(tag, &name);
        }
        self.hacks.insert(name, hack);
        Ok(())
    }

    fn load_filters(&mut self, filters_path: &Path) -> Result<(), StorageLoadError> {
        self.filters = Filters::load_from_path(filters_path)?;
        Ok(())
    }
}
