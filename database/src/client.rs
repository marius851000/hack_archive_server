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
        'main: for mut document in &mut potentially_conflicting.rows.into_iter() {
            let mut conflicts = document.get_conflicts_mut();
            //TODO: try to put some of this in common with the save conflict resolution
            while !conflicts.is_empty() {
                // conflict. Let's resolve it before answering
                let mut empty_conflicts = Vec::new();
                swap(&mut empty_conflicts, conflicts);
                let conflict_owned = empty_conflicts;

                let document_id = document.get_id().to_string();
                let mut logged_conflicts = Vec::new();

                let mut raw_docs: Vec<T> = Vec::new();
                for mut other_document in conflict_owned.into_iter() {
                    logged_conflicts.push(other_document.clone());
                    document.merge(&other_document);
                    other_document.mark_as_deleted();
                    raw_docs.push(other_document);
                }
                logged_conflicts.push(document.clone());
                raw_docs.push(document);

                self.log_handled_conflict(database.name(), &raw_docs, logged_conflicts)
                    .await?;

                for result in database.bulk_docs(&mut raw_docs).await? {
                    if let Err(e) = result {
                        if !matches!(e.status, StatusCode::CONFLICT | StatusCode::NOT_FOUND) {
                            return Err(HackClientError::InternalDBError(e));
                        }
                    }
                }
                let new_potentially_conflicting = database
                    .get_bulk_params::<T>(
                        vec![document_id],
                        Some(QueryParams::default().conflicts(true)),
                    )
                    .await?;

                document = match new_potentially_conflicting.total_rows {
                    0 => break 'main,
                    1 => new_potentially_conflicting.rows.get(0).unwrap().clone(),
                    _ => unreachable!(), //TODO:
                };

                conflicts = document.get_conflicts_mut();
            }
            resolved_document.push(document)
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
                    //Time for an infinite loop
                    //TODO: actually, include a loop counter to print warning after too much iteration
                    let id: DocumentId = document.get_id().into();
                    loop {
                        //1. get the conflicting document
                        let conflicting_documents = database
                            .get_bulk_params::<T>(
                                vec![id.clone()],
                                Some(QueryParams::default().conflicts(true)),
                            )
                            .await?;

                        let mut raw_docs = Vec::new();
                        let mut logged_conflicts = vec![document.clone()];

                        if let Some(mut conflicting_document) =
                            conflicting_documents.rows.into_iter().next()
                        {
                            //merge conflicting documents
                            let mut conflicts = Vec::new();
                            swap(&mut conflicts, conflicting_document.get_conflicts_mut());
                            for mut conflict_doc in conflicts {
                                logged_conflicts.push(conflict_doc.clone());
                                document.merge(&conflict_doc);
                                conflict_doc.mark_as_deleted();
                                raw_docs.push(conflict_doc);
                            }
                            logged_conflicts.push(conflicting_document.clone());
                            document.merge(&conflicting_document);
                            document.set_rev(conflicting_document.get_rev().as_ref());
                        }
                        raw_docs.push(document.clone());

                        self.log_handled_conflict(database.name(), &raw_docs, logged_conflicts)
                            .await?;

                        let mut have_problem = false;
                        for result in database.bulk_docs(&mut raw_docs).await? {
                            if let Err(e) = result {
                                if e.status == http::StatusCode::CONFLICT {
                                    have_problem = true;
                                    break;
                                } else {
                                    return Err(HackClientError::InternalDBError(e));
                                }
                            }
                        }
                        if !have_problem {
                            return Ok(());
                        }
                    }
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
