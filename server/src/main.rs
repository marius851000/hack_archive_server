use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use clap::Parser;
use database::HackClient;
//use database::MongoDriver;
//use mongodb::options::ClientOptions;
use pmd_hack_storage::{Query, Storage, Tag};
use server::pages::{css, file, hack, index, majority, oswald, tagged};
use server::AppData;
use std::{path::PathBuf, sync::Arc};

#[derive(Parser, Debug)]
#[clap()]
pub struct Opts {
    /// Path to the archive, should contain a hacks subfolder
    archive_folder: PathBuf,
    bind_address: String,
    /// base url, shouldn't end with /
    root_url: String,
    scope: String,
    couch_uri: String,
    couch_username: String,
    couch_password: String,
    /// feature flag to decide if majority token stuff should be visible on every page (doesn't completly disable it, it's just graphical)
    #[clap(short, long)]
    use_majority_token: bool,
}

#[tokio::main]
async fn main() {
    let opts = Opts::parse();

    let storage = Storage::load_from_folder(&opts.archive_folder).unwrap();
    println!("hacks loaded");

    let hidden_by_default = vec![
    (
        "Hacks marked being considered as being likely to be perceived as offensive".into(),
        Query::AtLeastOneOfTag(vec![Tag("likely-offensive".into())])
    ),
    (
        "Hacks marked as being explicitly refused in the SkyTemple hack list for moderation reason".into(),
        Query::AtLeastOneOfTag(vec![Tag("refused-skytemple".into())]),
    ),
    (
        "Merged hack".into(),
        Query::AtLeastOneOfTag(vec![Tag("deprecated".into())]),
    )
    ];

    let app_data = Arc::new(AppData {
        root_url: opts.root_url,
        storage,
        hidden_by_default,
        use_majority_token: opts.use_majority_token,
    });

    let hackclient = HackClient::new_from_connection_info(
        &opts.couch_uri,
        &opts.couch_username,
        &opts.couch_password,
    )
    .await
    .unwrap();

    println!("connected to couchdb");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_data.clone()))
            .app_data(Data::new(hackclient.clone()))
            .service(
                web::scope(&opts.scope)
                    .service(oswald)
                    .service(css::css)
                    .service(index::index)
                    .service(majority::majority)
                    .service(tagged::tagged)
                    .service(hack::hack)
                    .service(file::file),
            )
    })
    .bind(&opts.bind_address)
    .unwrap()
    .run()
    .await
    .unwrap();
}

//TODO: consider making language hack-wide, and do not tag these in the file
//TODO: sort tags
