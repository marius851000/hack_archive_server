mod common;
pub use common::*;

pub mod extractor;

pub mod pages;

pub mod message;

use extractor::RequestData;
use pmd_hack_storage::{Query, Storage, Tag};

pub struct AppData {
    pub root_url: String,
    pub storage: Storage,
    /// String: description of the reason
    /// Query: when does it match
    pub hidden_by_default: Vec<(String, Query)>,
    pub use_majority_token: bool,
}

impl AppData {
    pub fn route(&self, _request_data: &RequestData, path: &str) -> String {
        format!("{}/{}", self.root_url, path)
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
    use crate::{is_illegal_hack_slug, AppData};
    use pmd_hack_storage::Storage;

    #[test]
    pub fn test_app_data() {
        let app_data = AppData {
            root_url: "https://example.com".to_string(),
            storage: Storage::default(),
            hidden_by_default: vec![],
            use_majority_token: false,
        };
        assert_eq!(app_data.route_static("hello"), "https://example.com/hello");
    }

    #[test]
    pub fn test_illegal_hack_slug() {
        assert!(is_illegal_hack_slug("I/am"));
        assert!(is_illegal_hack_slug("faq"));
        assert!(!is_illegal_hack_slug("hello"));
    }
}
