use serde::Deserialize;
use serde_with::{serde_as, OneOrMany};

use std::{
    collections::{HashMap, HashSet},
    fs::{metadata, read_dir, File},
    io,
    path::PathBuf,
};
use thiserror::Error;

use crate::TagInfo;

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

#[serde_as]
#[derive(Deserialize)]
pub struct HackFile {
    pub label: String,
    #[serde_as(deserialize_as = "OneOrMany<_>")]
    #[serde(default)]
    pub language: Vec<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub base: Option<String>,
    pub filename: String,
}

#[derive(Error, Debug)]
pub enum HackLoadError {
    #[error("Can't open the file containing hack data in {1:?}")]
    CantOpenFile(#[source] io::Error, PathBuf),
    #[error("Can't read or parse the file {1:?} as a hack data file")]
    CantParseReadFile(#[source] serde_json::Error, PathBuf),
}

pub struct Hack {
    pub data: HackData,
    pub implied_tags: HashSet<Tag>,
    pub folder: PathBuf,
}

impl Hack {
    pub fn load_from_folder(folder: PathBuf, taginfo: &TagInfo) -> Result<Self, HackLoadError> {
        let hack_data_path = folder.join("hack.json");
        let json_file = File::open(&hack_data_path)
            .map_err(|e| HackLoadError::CantOpenFile(e, hack_data_path.clone()))?;
        let data: HackData = serde_json::from_reader(json_file)
            .map_err(|e| HackLoadError::CantParseReadFile(e, hack_data_path.clone()))?;

        // add implied tags, ensuring there are no infinite loops and it is recursive
        let mut implied_tags = HashSet::new();
        let mut all_tags = data.tags.clone();
        let mut stack_to_manage: Vec<Tag> = all_tags.iter().map(|x| x.clone()).collect();

        while let Some(tag) = stack_to_manage.pop() {
            if let Some(info_for_tag) = taginfo.tags.get(&tag) {
                for implied_tag in &info_for_tag.implies {
                    if all_tags.get(&implied_tag).is_some() {
                        continue;
                    }
                    implied_tags.insert(implied_tag.clone());
                    all_tags.insert(implied_tag.clone());
                    stack_to_manage.push(implied_tag.clone())
                }
            }
        }
        Ok(Self {
            data,
            folder,
            implied_tags,
        })
    }

    pub fn check_files(&self) {
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
                panic!("{:?} isn't a file", file.path());
            };
            let file_name = file.file_name().to_str().unwrap().to_string();
            if file_name == "index.html" {
                continue;
            }
            if referenced_files.contains(&file_name) {
                referenced_files.remove(&file_name);
            } else {
                panic!("{:?} doesn't exist, while being referenced", file.path());
            }
        }

        if !referenced_files.is_empty() {
            panic!(
                "the hack at {:?} references the files {:?} while there are non-existant",
                self.folder, referenced_files
            );
        }
    }

    pub fn all_tags(&self) -> HashSet<Tag> {
        let mut r = self.data.tags.clone();
        r.extend(self.implied_tags.clone());
        r
    }
}
