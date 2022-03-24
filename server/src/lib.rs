mod common;
pub use common::*;

pub mod extractor;

pub mod pages;

pub mod message;

use pmd_hack_storage::{Query, Storage};

pub struct AppData {
    pub root_url: String,
    pub storage: Storage,
    /// String: description of the reason
    /// Query: when does it match
    pub hidden_by_default: Vec<(String, Query)>,
    pub use_majority_token: bool,
}

/// Return true if the hack id is illegal (due to being reserved/user for a page)
pub fn is_illegal_hack_slug(name: &str) -> bool {
    if name.contains('/') {
        true
    } else {
        match name {
            //already user
            "Oswald-Medium.ttf" => true,
            "style.css" => true,
            "majority" => true,
            "tagged" => true,

            //likely to be used
            "faq" => true,
            "information" => true,
            "search" => true,
            "filter" => true,
            "download" => true,
            "marius" => true,

            //too generic name
            "pokemon-mystery-dungeon" => true,
            "explorers" => true,
            "super" => true,
            "gates" => true,

            _ => true,
        }
    }
}
