mod common;
pub use common::*;

pub mod extractor;

pub mod pages;

pub mod message;

use extractor::RequestData;
use fluent_templates::ArcLoader;
use pmd_hack_storage::{Query, Storage, Tag};
use qstring::QString;

pub struct AppData {
    pub root_url: String,
    pub storage: Storage,
    /// String: description of the reason
    /// Query: when does it match
    pub hidden_by_default: Vec<(String, Query)>,
    pub use_majority_token: bool,
    pub locales: ArcLoader,
}

impl AppData {
    pub fn route(&self, request_data: &RequestData, path: &str) -> String {
        let qs = QString::new(vec![(
            "lang".to_string(),
            request_data.language.to_string(),
        )]);
        format!("{}/{}?{}", self.root_url, path, qs)
    }

    pub fn route_static(&self, path: &str) -> String {
        format!("{}/{}", self.root_url, path)
    }

    pub fn route_hack_file(&self, hack_slug: &str, hack_file: &str) -> String {
        format!("{}/{}/{}", self.root_url, hack_slug, hack_file)
    }

    pub fn route_hack(&self, request_data: &RequestData, hack_slug: &str) -> String {
        self.route(request_data, hack_slug)
    }

    pub fn route_hack_list_by_tag(&self, request_data: &RequestData, tag: &Tag) -> String {
        self.route(request_data, &format!("tagged/{}", tag.0))
    }

    pub fn route_index_root(&self) -> String {
        self.route_static("index")
    }

    pub fn route_taginfo_file(&self) -> String {
        self.route_static("index/taginfo.json")
    }

    pub fn route_index_hacks(&self) -> String {
        self.route_static("index/hacks")
    }

    pub fn route_index_hack(&self, hack_slug: &str) -> String {
        self.route_static(&format!("index/hacks/{}", hack_slug))
    }
}

/// Return true if the hack id is illegal (due to being reserved/user for a page)
/// Currently unused
pub fn is_illegal_hack_slug(name: &str) -> bool {
    if name.contains('/') {
        true
    } else {
        matches!(
            name,
            //already user
            "Oswald-Medium.ttf"|
            "style.css"|
            "majority"|
            "tagged"|
            "create_majority_token"|
            "disconnect_majority_token"|
            "connect_majority_token"|
            "index"|

            //likely to be used
            "faq"|
            "information"|
            "search"|
            "filter"|
            "download"|
            "marius"|

            //too generic name
            "pokemon-mystery-dungeon"|
            "adventurers"|
            "explorers"|
            "super"|
            "gates"
        )
    }
}

#[cfg(test)]
mod test {
    use crate::is_illegal_hack_slug;

    #[test]
    pub fn test_illegal_hack_slug() {
        assert!(is_illegal_hack_slug("I/am"));
        assert!(is_illegal_hack_slug("faq"));
        assert!(!is_illegal_hack_slug("hello"));
    }
}
