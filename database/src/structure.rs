use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

/// Token representing a major token.
#[derive(Debug, Serialize, Deserialize)]
pub struct MajorityCheck {
    /// Who certified this user
    pub sources: Vec<ObjectId>,
    /// If this require to have at least one source
    pub require_source: bool,
    /// Arbitrary comment
    pub comment: String,
    /// If it can certify another user
    pub can_perform_check: bool,
    /// A secret token, owned by the major person
    pub token: String,
}

impl MajorityCheck {
    pub fn db_name() -> &'static str {
        "majority_check"
    }
}
