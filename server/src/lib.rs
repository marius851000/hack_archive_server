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
}
