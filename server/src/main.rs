use actix_web::{web, App, HttpServer};
use clap::Parser;
use database::MongoDriver;
use mongodb::options::ClientOptions;
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
    mongo_connection_string: String,
}

#[actix_web::main]
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
    });

    let client_options = ClientOptions::parse(&opts.mongo_connection_string)
        .await
        .unwrap();
    let client = mongodb::Client::with_options(client_options).unwrap();

    let db = client.database("archivedb");
    let driver = MongoDriver::new(db).await;

    println!("connected to mongodb");

    HttpServer::new(move || {
        App::new()
            .data(app_data.clone())
            .data(driver.clone())
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
