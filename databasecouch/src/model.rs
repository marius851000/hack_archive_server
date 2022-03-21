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
}

impl Mergeable for MajorityToken {
    fn merge(&mut self, other: &Self) {
        self.certify.extend(other.certify.iter().cloned());
        self.admin_flags.merge(&other.admin_flags);
    }

    fn mark_as_deleted(&mut self) {
        self._deleted = Some(true);
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
