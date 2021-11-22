use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::File,
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::Tag;

#[derive(Debug, Error)]
pub enum TagInfoLoadError {
    #[error("Can't open the taginfo file at {1:?}")]
    CantOpenFile(#[source] io::Error, PathBuf),
    #[error("Can't read or parse the file {1:?} as a taginfo file")]
    CantParseReadFile(#[source] serde_json::Error, PathBuf),
}

#[derive(Deserialize)]
pub struct TagInfo {
    pub tags: HashMap<Tag, SingleTagInfo>,
    pub categories: HashMap<String, CategoryInfo>,
}

impl TagInfo {
    pub fn load_from_path(path: &Path) -> Result<Self, TagInfoLoadError> {
        let json_file =
            File::open(&path).map_err(|e| TagInfoLoadError::CantOpenFile(e, path.to_path_buf()))?;
        let result = serde_json::from_reader(json_file)
            .map_err(move |e| TagInfoLoadError::CantParseReadFile(e, path.to_path_buf()))?;
        Ok(result)
    }

    pub fn get_category_for_hack(&self, hack: &Tag) -> Option<&CategoryInfo> {
        if let Some(tag) = self.get_tag(hack) {
            if let Some(category) = self.categories.get(&tag.category) {
                println!("code !");
                Some(category)
            } else {
                println!("category not found");
                None
            }
        } else {
            println!("tag not found");
            None
        }
    }

    pub fn get_tag(&self, tag_id: &Tag) -> Option<&SingleTagInfo> {
        self.tags.get(tag_id)
    }
}

#[derive(Deserialize)]
pub struct SingleTagInfo {
    category: String,
}

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub struct CategoryInfo {
    pub background_color: String,
    pub border_color: String,
}
