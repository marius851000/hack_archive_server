use std::{error::Error, fmt::Display, mem::swap};

use couch_rs::{
    database::Database,
    document::TypedCouchDocument,
    error::CouchError,
    types::{
        document::{DocumentCreatedDetails, DocumentId},
        query::QueryParams,
    },
    Client,
};
use http::StatusCode;
use serde_json::json;
use uuid::Uuid;

use crate::{get_timestamp, model::MajorityToken, Mergeable};

#[derive(Debug)]
pub enum HackClientError {
    InternalDBError(CouchError),
}

impl Error for HackClientError {}

impl Display for HackClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InternalDBError(e) => write!(
                f,
                "Internal error while communcating with the dabase: {}",
                e
            ),
        }
    }
}

impl From<CouchError> for HackClientError {
    fn from(e: CouchError) -> Self {
        Self::InternalDBError(e)
    }
}

impl HackClientError {
    pub fn end_user_error_message(&self) -> String {
        match self {
            Self::InternalDBError(_) => "there has been an internal database error".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct HackClient {
    majority_token: Database,
    /// A database logging resolution of conflict, for debugging purpose. Is only written to.
    conflict_log: Database,
}

impl HackClient {
    pub async fn new(db_client: Client) -> Result<Self, HackClientError> {
        Ok(Self {
            majority_token: db_client.db("majority_token").await?,
            conflict_log: db_client.db("conflict_log").await?,
        })
    }

    pub async fn new_from_connection_info(
        uri: &str,
        username: &str,
        password: &str,
    ) -> Result<Self, HackClientError> {
        let client = Client::new(uri, username, password)?;
        Self::new(client).await
    }

    async fn log_handled_conflict<T: TypedCouchDocument>(
        &self,
        db_name: &str,
        raw_docs: &[T],
        conflicting_docs: Vec<T>,
    ) -> Result<DocumentCreatedDetails, HackClientError> {
        Ok(self
            .conflict_log
            .save(&mut json!(
                {
                    "_id": Uuid::new_v4(),
                    "db": db_name,
                    "timestamp": get_timestamp(),
                    "raw_docs": raw_docs,
                    "conflicting_docs": conflicting_docs
                }
            ))
            .await?)
    }

    /// Resolve a conflict.
    /// if to_write is None, solve the conflict in the database, and return the result
    /// if to_write is Some(object), it will insert the object in the database, handling potential conflict, and return the inserted
    /// (may also return None if it actually didn't existed)
    async fn handle_conflict<T: TypedCouchDocument + Mergeable>(
        &self,
        database: &Database,
        document_id: DocumentId,
        to_write: Option<T>,
    ) -> Result<Option<T>, HackClientError> {
        //TODO: actually, include a loop counter to print warning after too much iteration
        'main: loop {
            //1. get conflicting documents
            let new_potentially_conflicting = database
                .get_bulk_params::<T>(
                    vec![document_id.clone()],
                    Some(QueryParams::default().conflicts(true)),
                )
                .await?;

            //2. get the document if it exist
            let mut base_document =
                if let Some(d) = new_potentially_conflicting.rows.into_iter().next() {
                    d
                } else {
                    return Ok(None);
                };

            //3. get the list of conflicts
            let mut conflicts = Vec::new();
            swap(&mut conflicts, base_document.get_conflicts_mut());
            if conflicts.is_empty() && to_write.is_none() {
                return Ok(Some(base_document));
            }

            if let Some(to_write) = &to_write {
                conflicts.push(base_document);
                base_document = to_write.clone();
            }

            let mut logged_conflicts = vec![base_document.clone()];

            let mut raw_docs: Vec<T> = Vec::new();
            //TODO: the different part with write and read will be here
            //4. merge the conflicting documents
            for mut other_document in conflicts.into_iter() {
                logged_conflicts.push(other_document.clone());
                base_document.merge(&other_document);
                other_document.mark_as_deleted();
                raw_docs.push(other_document);
            }

            //Write specific section

            raw_docs.push(base_document.clone());
            logged_conflicts.push(base_document.clone());

            self.log_handled_conflict(database.name(), &raw_docs, logged_conflicts)
                .await?;

            for result in database.bulk_docs(&mut raw_docs).await? {
                if let Err(e) = result {
                    if !matches!(e.status, StatusCode::CONFLICT | StatusCode::NOT_FOUND) {
                        return Err(HackClientError::InternalDBError(e));
                    } else if e.status == StatusCode::CONFLICT {
                        continue 'main;
                    }
                }
            }

            return Ok(Some(base_document));
        }
    }

    async fn get_and_resolve_conflict_one<T: TypedCouchDocument + std::fmt::Debug + Mergeable>(
        &self,
        database: &Database,
        id: DocumentId,
    ) -> Result<Option<T>, HackClientError> {
        Ok(self
            .get_and_resolve_conflict(database, vec![id])
            .await?
            .into_iter()
            .next())
    }

    async fn get_and_resolve_conflict<T: TypedCouchDocument + std::fmt::Debug + Mergeable>(
        &self,
        database: &Database,
        ids: Vec<DocumentId>,
    ) -> Result<Vec<T>, HackClientError> {
        let potentially_conflicting = database
            .get_bulk_params::<T>(ids.clone(), Some(QueryParams::default().conflicts(true)))
            .await?;

        let mut resolved_document: Vec<T> = Vec::new();
        for mut document in &mut potentially_conflicting.rows.into_iter() {
            let conflicts = document.get_conflicts_mut();
            if !conflicts.is_empty() {
                if let Some(r) = self
                    .handle_conflict(database, document.get_id().into(), None)
                    .await?
                {
                    resolved_document.push(r);
                }
            } else {
                resolved_document.push(document);
            }
        }
        Ok(resolved_document)
    }

    async fn save_and_resolve_conflict<T: TypedCouchDocument + std::fmt::Debug + Mergeable>(
        &self,
        database: &Database,
        mut document: T,
    ) -> Result<(), HackClientError> {
        match database.save(&mut document).await {
            Ok(_) => Ok(()),
            Err(e) => {
                if e.status == http::StatusCode::CONFLICT {
                    //Damn ! A conflict
                    let id: DocumentId = document.get_id().into();
                    self.handle_conflict(database, id, Some(document)).await?;
                    Ok(())
                } else {
                    //Well, an error. Guess that happen sometimes...
                    Err(HackClientError::InternalDBError(e))
                }
            }
        }
    }

    /// If the _rev value is defined, update the token, otherwise, attempt to create it
    pub async fn save_majority_token(&self, token: MajorityToken) -> Result<(), HackClientError> {
        self.save_and_resolve_conflict(&self.majority_token, token)
            .await
    }

    pub async fn get_majority_token(
        &self,
        password: &str,
    ) -> Result<Option<MajorityToken>, HackClientError> {
        self.get_and_resolve_conflict_one(&self.majority_token, password.into())
            .await
    }
}
