use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::File,
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;

use super::Hack;

#[derive(Deserialize, Clone, Debug)]
pub struct Filter {
    pub label: String,
    #[serde(default)]
    pub hide: Vec<String>,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            hide: vec!["suggestive".into(), "explicit".into()],
            label: "Default".into(),
        }
    }
}

impl Filter {
    pub fn pass_filter(&self, hack: &Hack) -> bool {
        for keyword_to_disallow in &self.hide {
            if hack.data.tags.contains(keyword_to_disallow) {
                return false;
            }
        }
        true
    }
}

#[derive(Error, Debug)]
pub enum FiltersLoadError {
    #[error("Can't open file at {1:?}")]
    CantOpenFile(#[source] io::Error, PathBuf),
    #[error("Can't load filter from {1:?}")]
    CantLoadFilters(#[source] serde_json::Error, PathBuf),
}

#[derive(Deserialize)]
pub struct Filters {
    pub filters: HashMap<String, Filter>,
    pub default: String,
}

impl Default for Filters {
    fn default() -> Self {
        let mut filters = HashMap::new();
        filters.insert("default".into(), Filter::default());
        Self {
            filters,
            default: "default".to_string(),
        }
    }
}

impl Filters {
    pub fn load_from_path(path: &Path) -> Result<Self, FiltersLoadError> {
        let mut file =
            File::open(path).map_err(|e| FiltersLoadError::CantOpenFile(e, path.to_path_buf()))?;

        let result = serde_json::from_reader::<_, Self>(&mut file)
            .map_err(|e| FiltersLoadError::CantLoadFilters(e, path.to_path_buf()))?;

        Ok(result)
    }

    pub fn get_default(&self) -> Filter {
        self.filters
            .get(&self.default)
            .map(|x| x.clone())
            .unwrap_or(Filter::default())
    }

    pub fn get(&self, slug: &str) -> Option<Filter> {
        self.filters.get(slug).map(|x| x.clone())
    }
}
