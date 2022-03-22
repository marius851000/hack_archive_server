use std::collections::BTreeSet;

use couch_rs::Client;
use databasecouch::{model::MajorityToken, FieldWithTime, HackClient};

#[tokio::main]
async fn main() {
    let db_client = Client::new("http://127.0.0.1:5984", "admin", "test").unwrap();
    let hackclient = HackClient::new(db_client).await.unwrap();

    hackclient
        .save_majority_token(MajorityToken {
            _id: "testpassword".to_string(),
            _rev: "".to_string(),
            certify: BTreeSet::new(),
            admin_flags: FieldWithTime::new(databasecouch::model::MajorityTokenAdminFlags {
                can_certify: true,
                need_certification: false,
                revoked: false,
            }),
            _deleted: None,
            _conflicts: Vec::new(),
        })
        .await
        .unwrap();

    println!(
        "{:?}",
        hackclient.get_majority_token("testpassword").await.unwrap()
    );
}
