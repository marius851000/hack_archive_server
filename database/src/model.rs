use std::collections::BTreeSet;

use couch_rs::document::TypedCouchDocument;
use couch_rs::{types::document::DocumentId, CouchDocument};
use serde::{Deserialize, Serialize};

use crate::{FieldWithTime, Mergeable};

#[derive(Serialize, Deserialize, CouchDocument, Debug, Clone)]
pub struct MajorityToken {
    /// The password of the token. Also the primary key.
    pub _id: DocumentId,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub _rev: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _deleted: Option<bool>,
    /// The list of user this majority token has certified
    // Assumed to be append-only
    pub certify: BTreeSet<String>,
    pub admin_flags: FieldWithTime<MajorityTokenAdminFlags>,
    #[serde(default = "u64::default")]
    pub latest_certification_timestamp: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default = "Vec::default")]
    pub _conflicts: Vec<MajorityToken>,
}

impl Mergeable for MajorityToken {
    fn merge(&mut self, other: &Self) {
        self.certify.extend(other.certify.iter().cloned());
        self.admin_flags.merge(&other.admin_flags);
        self.admin_flags.0.revoked |= other.admin_flags.0.revoked;
        if other.latest_certification_timestamp > self.latest_certification_timestamp {
            self.latest_certification_timestamp = other.latest_certification_timestamp;
        };
    }

    fn mark_as_deleted(&mut self) {
        self._deleted = Some(true);
    }

    fn get_conflicts_mut(&mut self) -> &mut Vec<Self> {
        &mut self._conflicts
    }
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct MajorityTokenAdminFlags {
    /// Whether the list of user certified should be ignored when determining certified user
    pub can_certify: bool,
    /// Whether this token need certification to be valid
    pub need_certification: bool,
    /// Whether this token is valid
    pub revoked: bool,
}
