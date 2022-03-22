mod client;
pub use client::{HackClient, HackClientError};

pub mod model;

mod field_with_time;
pub use field_with_time::FieldWithTime;

/// Merge two couchdb document, but should leave _id, _rev and _deleted unchanged
pub trait Mergeable: Sized + Clone {
    fn merge(&mut self, other: &Self);

    fn mark_as_deleted(&mut self);

    fn get_conflicts_mut(&mut self) -> &mut Vec<Self>;
}
