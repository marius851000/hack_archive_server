use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

#[derive(Deserialize, Serialize, Hash, PartialEq, Eq, Clone, Debug, PartialOrd, Ord)]
pub struct Tag(pub String);

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Default)]
pub struct Tags {
    pub tag_list: HashMap<Tag, HashSet<String>>,
}

impl Tags {
    pub fn add_hack_with_tag(&mut self, tag: &Tag, hack_slug: String) {
        match self.tag_list.get_mut(tag) {
            Some(hack_set) => {
                hack_set.insert(hack_slug);
            }
            None => {
                let mut new_hack_set = HashSet::new();
                new_hack_set.insert(hack_slug);
                self.tag_list.insert(tag.clone(), new_hack_set);
            }
        }
    }

    pub fn get_hack_for_tag(&self, tag: &Tag) -> Option<&HashSet<String>> {
        self.tag_list.get(tag)
    }
}
