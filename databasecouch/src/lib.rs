#![feature(map_first_last)]
mod client;
pub use client::{HackClient, HackClientError};

pub mod model;

mod field_history;
pub use field_history::FieldWithTime;

/// Merge two couchdb document, but should leave _id, _rev and _deleted unchanged
pub trait Mergeable: Sized + Clone {
    fn merge(&mut self, other: &Self);

    fn mark_as_deleted(&mut self);
}
