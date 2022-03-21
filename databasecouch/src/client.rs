use std::{error::Error, fmt::Display};

use couch_rs::{
    database::Database,
    document::TypedCouchDocument,
    error::CouchError,
    types::{document::DocumentId, query::QueryParams},
    Client,
};

use crate::{model::MajorityToken, Mergeable};

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

pub struct HackClient {
    majority_token: Database,
}

impl HackClient {
    pub async fn new(db_client: Client) -> Result<Self, HackClientError> {
        Ok(Self {
            majority_token: db_client.db("majority_token").await?,
        })
    }

    async fn get_and_resolve_conflict<T: TypedCouchDocument + std::fmt::Debug + Mergeable>(
        &self,
        database: &Database,
        id: DocumentId,
    ) -> Result<Option<T>, HackClientError> {
        let mut potentially_conflicting = database
            .get_bulk_params::<T>(
                vec![id.clone()],
                Some(QueryParams::default().conflicts(true)),
            )
            .await?;

        match potentially_conflicting.total_rows {
            0 => Ok(None),
            1 => Ok(Some(potentially_conflicting.rows.get(0).unwrap().clone())),
            _ => {
                // conflict. Let's resolve it before answering
                loop {
                    let mut document = potentially_conflicting.rows.get(0).unwrap().clone();
                    let mut raw_docs = Vec::new();
                    for mut other_document in potentially_conflicting.rows.into_iter().skip(1) {
                        document.merge(&other_document);
                        other_document.mark_as_deleted();
                        raw_docs.push(other_document);
                    }
                    raw_docs.push(document.clone());

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
                        return Ok(Some(document));
                    }

                    potentially_conflicting = database
                        .get_bulk_params::<T>(
                            vec![id.clone()],
                            Some(QueryParams::default().conflicts(true)),
                        )
                        .await?;
                }
            }
        }
        /*

            loop {
                //1. get the conflicting document
                let conflicting = database
                    .get_bulk_params::<T>(
                        vec![id.clone()],
                        Some(QueryParams::default().conflicts(true)),
                    )
                    .await?;
                //3. resolve the conflict(s)
                document.merge(&conflicting.rows);
                //4. actually perform the bulk edit
                let mut raw_docs = Vec::new();
                for mut row in conflicting.rows {
                    row.mark_as_deleted();
                    raw_docs.push(row);
                }
                raw_docs.push(document.clone());
                let mut have_problem = false;
                for result in database.bulk_docs(&mut raw_docs).await? {
                    if let Err(e) = result {
                        if e.status == http::StatusCode::CONFLICT {
                            have_problem = true;
                            break;
                        } else {
                            return Err(HackClientError::InternalDBError(e))
                        }
                    }
                }
                if !have_problem {
                    return Ok(());
                }
            }
        */
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
                        let conflicting = database
                            .get_bulk_params::<T>(
                                vec![id.clone()],
                                Some(QueryParams::default().conflicts(true)),
                            )
                            .await?;
                        //3. resolve the conflict(s)
                        let mut raw_docs = Vec::new();
                        for mut row in conflicting.rows {
                            document.merge(&row);
                            row.mark_as_deleted();
                            raw_docs.push(row);
                        }

                        raw_docs.push(document.clone());
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
        self.get_and_resolve_conflict(&self.majority_token, password.into())
            .await
    }
}
