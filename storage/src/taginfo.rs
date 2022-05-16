use crate::Tag;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fs::File,
    io,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TagInfoLoadError {
    #[error("Can't open the taginfo file at {1:?}")]
    CantOpenFile(#[source] io::Error, PathBuf),
    #[error("Can't read or parse the file {1:?} as a taginfo file")]
    CantParseReadFile(#[source] serde_json::Error, PathBuf),
}

#[derive(Deserialize, Serialize, Default)]
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

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(&self)
    }

    pub fn get_category_for_tag_id(&self, tag_id: &Tag) -> Option<&CategoryInfo> {
        if let Some(tag) = self.get_tag(tag_id) {
            self.get_category_for_single_tag_info(tag, tag_id)
        } else {
            println!("tag info for {:?} not found", tag_id);
            None
        }
    }

    pub fn get_category_for_single_tag_info(
        &self,
        tag: &SingleTagInfo,
        tag_id: &Tag,
    ) -> Option<&CategoryInfo> {
        if let Some(category_id) = &tag.category {
            let result = self.categories.get(category_id);
            if result.is_none() {
                warn!(
                    "category {:?} for tag {:?} not found",
                    &tag.category, tag_id
                );
            };
            result
        } else {
            info!("tag info for {:?} doesn't have a category entry", tag_id);
            None
        }
    }

    pub fn get_tag(&self, tag_id: &Tag) -> Option<&SingleTagInfo> {
        self.tags.get(tag_id)
    }

    fn compare_tag(&self, a_id: &Tag, b_id: &Tag) -> Ordering {
        let a = self.get_tag(a_id);
        let b = self.get_tag(b_id);
        // sort by category
        let a_category_priority = a.map_or(0, |a| {
            self.get_category_for_single_tag_info(a, a_id)
                .map_or(0, |x| x.priority)
        });
        let b_catgeory_priority = b.map_or(0, |b| {
            self.get_category_for_single_tag_info(b, b_id)
                .map_or(0, |x| x.priority)
        });
        match a_category_priority.cmp(&b_catgeory_priority) {
            Ordering::Equal => (),
            other => return other.reverse(),
        };
        // sort by tag priority
        let a_priority = a.map_or(0, |x| x.priority);
        let b_piority = b.map_or(0, |x| x.priority);
        match a_priority.cmp(&b_piority) {
            Ordering::Equal => (),
            other => return other.reverse(),
        };
        // sort by label/tag name
        let a_label = a.map_or(&a_id.0, |x| x.label.as_ref().unwrap_or(&a_id.0));
        let b_label = b.map_or(&b_id.0, |x| x.label.as_ref().unwrap_or(&b_id.0));
        a_label.cmp(b_label)
    }

    pub fn orders_tags(&self, mut tags: Vec<Tag>) -> Vec<Tag> {
        tags.sort_unstable_by(|a, b| self.compare_tag(a, b));
        tags
    }

    pub fn get_implied_tags(&self, base_tags: &HashSet<Tag>) -> HashSet<Tag> {
        let mut implied_tags = HashSet::new();
        let mut all_tags = base_tags.clone();
        let mut stack_to_manage: Vec<Tag> = all_tags.iter().cloned().collect();

        while let Some(tag) = stack_to_manage.pop() {
            if let Some(info_for_tag) = self.tags.get(&tag) {
                for implied_tag in &info_for_tag.implies {
                    if all_tags.get(implied_tag).is_some() {
                        continue;
                    }
                    implied_tags.insert(implied_tag.clone());
                    all_tags.insert(implied_tag.clone());
                    stack_to_manage.push(implied_tag.clone())
                }
            }
        }

        all_tags
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SingleTagInfo {
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub implies: Vec<Tag>,
    #[serde(default)]
    pub priority: u32,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

fn is_false(b: &bool) -> bool {
    !b
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all(deserialize = "kebab-case"))]
pub struct CategoryInfo {
    pub background_color: String,
    pub border_color: String,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub required_for_file: bool,
    #[serde(default)]
    pub priority: u32,
}
