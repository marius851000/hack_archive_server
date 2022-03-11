use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MajorityCheck {
    pub sources: Vec<ObjectId>,
    pub require_source: bool,
    pub comment: String,
    pub can_perform_check: bool,
    pub token: String,
}

impl MajorityCheck {
    pub fn db_name() -> &'static str {
        "majority_check"
    }
}
