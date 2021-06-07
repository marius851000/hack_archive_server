use std::{collections::HashMap, fs::File, io, path::PathBuf};
use thiserror::Error;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct HackData {
  pub name: String,
  #[serde(default)]
  pub authors: Vec<String>,
  #[serde(default)]
  pub description: Option<String>,
  #[serde(default)]
  pub tags: Vec<String>,
  #[serde(default)]
  pub source: Option<String>,
  #[serde(default)]
  pub skytemple_db_id: Option<String>,
  #[serde(default)]
  pub screenshots: Vec<String>,
  #[serde(default)]
  pub links: HashMap<String, String>
}

#[derive(Deserialize)]
pub struct HackFile {
  pub label: String,
  #[serde(default)]
  pub language: String,
  #[serde(default)]
  pub description: String,
  pub filename: String,
}

#[derive(Error, Debug)]
pub enum HackLoadError {
  #[error("Can't open the file containing hack data in {1:?}")]
  CantOpenFile(#[source] io::Error, PathBuf),
  #[error("Can't read or parse the file {1:?} as a hack data file")]
  CantParseReadFile(#[source] serde_json::Error, PathBuf)
}

pub struct Hack {
  pub data: HackData,
  pub folder: PathBuf,
}

impl Hack {
  pub fn load_from_folder(folder: PathBuf) -> Result<Self, HackLoadError> {
    let hack_data_path = folder.join("hack.json");
    let json_file = File::open(&hack_data_path).map_err(|e| HackLoadError::CantOpenFile(e, hack_data_path.clone()))?;
    let data: HackData = serde_json::from_reader(json_file).map_err(|e| HackLoadError::CantParseReadFile(e, hack_data_path.clone()))?;
    Ok(Self {
      data,
      folder
    })
  }
}