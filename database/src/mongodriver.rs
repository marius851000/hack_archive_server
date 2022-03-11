use std::fmt::Display;

use mongodb::{
    bson::{doc, oid::ObjectId, Bson},
    error::WriteFailure,
    options::{FindOneOptions, IndexOptions},
    Database, IndexModel,
};
use rand::{distributions::Alphanumeric, Rng};

use crate::MajorityCheck;

#[derive(Debug)]
pub enum MongoDriverError {
    MongoError(mongodb::error::Error),
    DidntReturnObjectID(Bson),
}

impl Display for MongoDriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Self::MongoError(_) => write!(f, "database error"),
            Self::DidntReturnObjectID(_) => {
                write!(f, "error analyzing the result of a bdd request")
            }
        }
    }
}

impl std::error::Error for MongoDriverError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Self::MongoError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<mongodb::error::Error> for MongoDriverError {
    fn from(err: mongodb::error::Error) -> Self {
        Self::MongoError(err)
    }
}

fn bson_to_objectid(bson: Bson) -> Result<ObjectId, MongoDriverError> {
    if let Some(oid) = bson.as_object_id() {
        Ok(oid)
    } else {
        Err(MongoDriverError::DidntReturnObjectID(bson))
    }
}

#[derive(Clone)]
pub struct MongoDriver {
    db: Database,
}

impl MongoDriver {
    /// Panic :
    /// Will panic on any error. This is expected to be called during initialisation.
    pub async fn new(db: Database) -> Self {
        db.collection::<MajorityCheck>(MajorityCheck::db_name())
            .create_index(
                IndexModel::builder()
                    .keys(doc! {
                        "token": 1
                    })
                    .options({
                        let mut option = IndexOptions::default();
                        option.name = Some("unique_token".into());
                        option.unique = Some(true);
                        option
                    })
                    .build(),
                None,
            )
            .await
            .unwrap();
        Self { db }
    }

    pub async fn add_major_user(
        &self,
        sources: Vec<ObjectId>,
        require_source: bool,
        comment: String,
        can_perform_check: bool,
    ) -> Result<ObjectId, MongoDriverError> {
        let mut counter: u32 = 0;
        loop {
            let token = rand::thread_rng()
                .sample_iter(&Alphanumeric)
                .take(16)
                .map(char::from)
                .collect();
            counter += 1;
            let temp_result = self
                .db
                .collection::<MajorityCheck>(MajorityCheck::db_name())
                .insert_one(
                    MajorityCheck {
                        sources: sources.clone(),
                        comment: comment.clone(),
                        can_perform_check,
                        require_source,
                        token,
                    },
                    None,
                )
                .await;
            match temp_result {
                Ok(r) => return bson_to_objectid(r.inserted_id),
                Err(e) => match &*e.kind {
                    mongodb::error::ErrorKind::Write(WriteFailure::WriteError(write_error)) => {
                        if write_error.code == 11000 {
                            if counter > 100 {
                                return Err(e.into());
                            } else {
                                continue;
                            }
                        } else {
                            return Err(e.into());
                        }
                    }
                    _ => return Err(e.into()),
                },
            }
        }
    }

    pub async fn get_majority_check_by_token(
        &self,
        token: &str,
    ) -> Result<Option<MajorityCheck>, MongoDriverError> {
        Ok(self
            .db
            .collection::<MajorityCheck>(MajorityCheck::db_name())
            .find_one(
                doc! {
                    "token": token
                },
                FindOneOptions::default(),
            )
            .await?)
    }
}
