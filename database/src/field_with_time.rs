use serde::{Deserialize, Serialize};

use crate::get_timestamp;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FieldWithTime<T: PartialEq + std::fmt::Debug + Clone>(
    /// current value
    pub T,
    /// timestamp at which the current value was set
    pub u64,
);

impl<T: PartialEq + std::fmt::Debug + Clone> FieldWithTime<T> {
    pub fn new(value: T) -> Self {
        Self(value, get_timestamp())
    }

    pub fn update(&mut self, value: T) {
        self.0 = value;
        self.1 = get_timestamp();
    }

    pub fn get(&self) -> &T {
        &self.0
    }

    pub fn get_best_of_two(self, other: Self) -> Self {
        if other.1 >= self.1 {
            other
        } else {
            self
        }
    }

    pub fn merge(&mut self, other: &Self) {
        if &self.0 != &other.0 {
            if other.1 >= self.1 {
                self.1 = other.1;
                self.0 = other.0.clone();
            }
        }
    }

    pub fn get_best(first: Self, others: Vec<Self>) -> Self {
        let mut newest = first.1;
        let mut best = first;
        for other in others {
            if other.1 >= newest {
                newest = other.1;
                best = other;
            };
        }
        best
    }
}
