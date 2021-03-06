mod client;
use std::time::{Duration, SystemTime};

pub use client::{HackClient, HackClientError};

pub mod model;

mod field_with_time;
pub use field_with_time::FieldWithTime;

/// Merge two couchdb document, but should leave _id, _rev and _deleted unchanged
pub trait Mergeable: Sized + Clone {
    fn merge(&mut self, other: &Self);

    //TODO: for the last two, just provide some sort of struct or something
    fn mark_as_deleted(&mut self);

    fn get_conflicts_mut(&mut self) -> &mut Vec<Self>;
}

pub fn get_timestamp() -> u64 {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    time.as_secs()
}
