use serde::Deserialize;

use std::{
    collections::{BTreeMap, HashMap, HashSet},
    fs::{metadata, read_dir, File},
    io,
    path::PathBuf,
};
use thiserror::Error;

use crate::{taginfo::SingleTagInfo, TagInfo, MAJORONLY_CATEGORY};

use super::Tag;

#[derive(Deserialize)]
pub struct HackData {
    pub name: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: HashSet<Tag>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub skytemple_db_id: Option<String>,
    #[serde(default)]
    pub screenshots: Vec<String>,
    #[serde(default)]
    pub links: HashMap<String, String>,
    pub files: Vec<HackFile>,
}

#[derive(Deserialize)]
pub struct HackFile {
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: HashSet<Tag>,
    #[serde(default)]
    pub implied_tags: HashSet<Tag>,
    pub filename: String,
}

impl HackFile {
    pub fn get_all_tags(&self) -> HashSet<Tag> {
        let mut r = self.tags.clone();
        r.extend(self.implied_tags.clone());
        r
    }
}
#[derive(Error, Debug)]
pub enum HackLoadError {
    #[error("Can't open the file containing hack data in {1:?}")]
    CantOpenFile(#[source] io::Error, PathBuf),
    #[error("Can't read or parse the file {1:?} as a hack data file")]
    CantParseReadFile(#[source] serde_json::Error, PathBuf),
    #[error("The file {0:?} was listed by the hack, but is non-existant")]
    MissingFiles(String),
    #[error("The file at {0:?} is present in the hacks folder’s, but seems to be not referenced. All files here are public.")]
    UnreferencedFile(PathBuf),
    #[error("The thing at {0:?} is not a file (an hack folder should only contain files)")]
    FileNotFile(PathBuf),
}

pub struct Hack {
    pub data: HackData,
    pub implied_tags: HashSet<Tag>,
    pub folder: PathBuf,
}

impl Hack {
    pub fn load_from_folder(
        folder: PathBuf,
        taginfo: &TagInfo,
    ) -> Result<(Self, Vec<HackLoadError>), HackLoadError> {
        let hack_data_path = folder.join("hack.json");
        let json_file = File::open(&hack_data_path)
            .map_err(|e| HackLoadError::CantOpenFile(e, hack_data_path.clone()))?;
        let mut data: HackData = serde_json::from_reader(json_file)
            .map_err(|e| HackLoadError::CantParseReadFile(e, hack_data_path.clone()))?;

        // add implied tags, ensuring there are no infinite loops and it is recursive
        let implied_tags = taginfo.get_implied_tags(&data.tags);
        for file in data.files.iter_mut() {
            file.implied_tags = taginfo.get_implied_tags(&file.tags);
        }

        let result = Self {
            data,
            folder,
            implied_tags,
        };
        let non_fatal_errors = result.check_files();
        Ok((result, non_fatal_errors))
    }

    /// Check for missing files.
    pub fn check_files(&self) -> Vec<HackLoadError> {
        let mut errors = Vec::new();

        let mut referenced_files = HashSet::new();
        for screenshot in &self.data.screenshots {
            referenced_files.insert(screenshot.to_string());
        }
        for file in &self.data.files {
            referenced_files.insert(file.filename.to_string());
        }
        referenced_files.insert("hack.json".into());

        for file in read_dir(&self.folder).unwrap().map(|e| e.unwrap()) {
            let metadata = metadata(file.path()).unwrap();
            if !metadata.is_file() {
                errors.push(HackLoadError::FileNotFile(file.path()))
            };
            let file_name = file.file_name().to_str().unwrap().to_string();
            if file_name == "index.html" {
                continue;
            }
            if referenced_files.contains(&file_name) {
                referenced_files.remove(&file_name);
            } else {
                errors.push(HackLoadError::UnreferencedFile(file.path()));
            }
        }

        for missing_file in referenced_files.into_iter() {
            errors.push(HackLoadError::MissingFiles(missing_file));
        }

        errors
    }

    pub fn all_tags(&self) -> HashSet<Tag> {
        let mut r = self.data.tags.clone();
        r.extend(self.implied_tags.clone());
        for files in &self.data.files {
            r.extend(files.implied_tags.clone());
        }
        r
    }

    pub fn get_major_only_tags<'a>(
        &self,
        taginfo: &'a TagInfo,
    ) -> BTreeMap<Tag, &'a SingleTagInfo> {
        let mut r = BTreeMap::new();
        for tag_id in &self.all_tags() {
            if let Some(tag) = taginfo.get_tag(tag_id) {
                if tag.category.as_deref() == Some(MAJORONLY_CATEGORY) {
                    r.insert(tag_id.clone(), tag);
                }
            }
        }
        r
    }

    pub fn need_majority_token(&self, taginfo: &TagInfo) -> bool {
        !self.get_major_only_tags(taginfo).is_empty()
    }
}
