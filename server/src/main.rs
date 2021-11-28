use actix_web::{App, HttpServer, web};
use clap::Parser;
use pmd_hack_storage::{Query, Storage, Tag};
use server::{AppData, css_page, file_page, hack_page, index_page, oswald, tagged_page};
use std::{path::PathBuf, sync::Arc};

#[derive(Parser, Debug)]
#[clap()]
pub struct Opts {
    /// Path to the archive, should contain a hacks subfolder
    archive_folder: PathBuf,
    bind_address: String,
    /// base url, shouldn't end with /
    root_url: String,
    scope: String
}

#[actix_web::main]
async fn main() {
    let opts = Opts::parse();

    let storage = Storage::load_from_folder(&opts.archive_folder).unwrap();
    println!("hacks loaded");

    let hidden_by_default = vec![(
        "Hacks marked as being explicitly refused for the SkyTemple hack list for moderation reason".into(),
        Query::AtLeastOneOfTag(vec![Tag("refused-skytemple".into())]),
    )];

    let app_data = Arc::new(AppData {
        root_url: opts.root_url,
        storage,
        hidden_by_default,
    });

    HttpServer::new(move || {
        App::new()
            .data(app_data.clone())
            .service(
                web::scope(&opts.scope)
                    .service(oswald)
                    .service(css_page)
                    .service(index_page)
                    .service(tagged_page)
                    .service(hack_page)
                    .service(file_page)
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
