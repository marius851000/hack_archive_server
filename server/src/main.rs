use actix_web::{App, HttpServer};
use clap::Parser;
use pmd_hack_storage::{Query, Storage, Tag};
use server::{css_page, file_page, hack_page, index_page, oswald, AppData};
use std::{path::PathBuf, sync::Arc};

#[derive(Parser, Debug)]
#[clap()]
pub struct Opts {
    /// Path to the archive, should contain a hacks subfolder
    archive_folder: PathBuf,
    bind_address: String,
    /// base url, shouldn't end with /
    root_url: String,
}

#[actix_web::main]
async fn main() {
    let opts = Opts::parse();

    let storage = Storage::load_from_folder(&opts.archive_folder).unwrap();
    println!("hacks loaded");

    let hidden_by_default = vec![
        (
            "Hacks marked as suggestive".into(),
            Query::AtLeastOneOfTag(vec![Tag("suggestive".into())]),
        ),
        (
            "Hacks marked as explicit".into(),
            Query::AtLeastOneOfTag(vec![Tag("explicit".into())]),
        ),
    ];

    let app_data = Arc::new(AppData {
        root_url: opts.root_url,
        storage,
        hidden_by_default,
    });

    HttpServer::new(move || {
        App::new()
            .data(app_data.clone())
            .service(oswald)
            .service(css_page)
            .service(index_page)
            .service(hack_page)
            .service(file_page)
    })
    .bind(&opts.bind_address)
    .unwrap()
    .run()
    .await
    .unwrap();
}
