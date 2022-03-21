use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

fn get_timestamp() -> u64 {
    let time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(Duration::ZERO);
    time.as_secs()
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FieldWithTime<T: PartialEq + std::fmt::Debug + Clone> {
    /// current value
    value: T,
    /// timestamp at which the current value was set
    timestamp_set: u64,
}

impl<T: PartialEq + std::fmt::Debug + Clone> FieldWithTime<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            timestamp_set: get_timestamp(),
        }
    }

    pub fn update(&mut self, value: T) {
        self.value = value;
        self.timestamp_set = get_timestamp();
    }

    pub fn get(&self) -> &T {
        &self.value
    }

    pub fn get_best_of_two(self, other: Self) -> Self {
        if other.timestamp_set >= self.timestamp_set {
            other
        } else {
            self
        }
    }

    pub fn merge(&mut self, other: &Self) {
        if other.timestamp_set >= self.timestamp_set {
            self.timestamp_set = other.timestamp_set;
            self.value = other.value.clone();
        }
    }

    pub fn get_best(first: Self, others: Vec<Self>) -> Self {
        let mut newest = first.timestamp_set;
        let mut best = first;
        for other in others {
            if other.timestamp_set >= newest {
                newest = other.timestamp_set;
                best = other;
            };
        }
        best
    }
}
