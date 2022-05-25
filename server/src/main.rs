use actix_web::web::Data;
use actix_web::{web, App, HttpServer};
use clap::Parser;
use database::HackClient;
use fluent_templates::ArcLoader;
//use database::MongoDriver;
//use mongodb::options::ClientOptions;
use pmd_hack_storage::{Query, Storage, Tag};
use server::pages::{
    create_majority_token, css, disconnect_majority_token, file, hack, hackindex, index, majority,
    oswald, tagged,
};
use server::AppData;
use std::path::PathBuf;
use unic_langid::langid;
use url::Url;

#[derive(Parser, Debug)]
#[clap()]
pub struct Opts {
    /// Path to the archive, should contain a hacks subfolder
    archive_folder: PathBuf,
    locales_folder: PathBuf,
    bind_address: String,
    /// base url, shouldn't end with /
    root_url: String,
    scope: String,
    couch_uri: String,
    couch_username: String,
    couch_password: String,
    /// experimental feature flag to decide if majority token stuff should be visible on every page (doesn't completly disable it, it just hide it)
    #[clap(short, long)]
    use_majority_token: bool,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let opts = Opts::parse();

    let root_url = Url::parse(&opts.root_url).unwrap();
    if root_url.cannot_be_a_base() {
        panic!("The provided url ({:?}) cannot be use a base url", root_url);
    }

    let locales = ArcLoader::builder(&opts.locales_folder, langid!("en"))
        .shared_resources(Some(&[opts.locales_folder.join("core.ftl")]))
        .build()
        .unwrap();

    let storage = Storage::load_from_folder(&opts.archive_folder).unwrap();
    storage.warn_missing_tags();

    println!("hacks loaded");

    //TODO: rework how hacks are hidden
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
        ),
        (
            "Hacks marked as being reserved to major person (contain pornography)".into(),
            Query::AtLeastOneOfTag(vec![Tag("pornographic".into())]),
        )
    ];

    let app_data = Data::new(AppData {
        root_url,
        storage,
        hidden_by_default,
        use_majority_token: opts.use_majority_token,
        locales,
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
            .app_data(app_data.clone())
            .app_data(Data::new(hackclient.clone()))
            .service(
                web::scope(&opts.scope)
                    .service(oswald)
                    .service(css::css)
                    .service(index::index)
                    .service(hackindex::index_root::index_root)
                    .service(hackindex::index_taginfo::index_taginfo)
                    .service(hackindex::index_hacks::index_hacks)
                    .service(hackindex::index_hack::index_hack)
                    .service(majority::majority)
                    .service(create_majority_token::create_majority_token)
                    .service(tagged::tagged)
                    .service(hack::hack)
                    .service(file::file)
                    .service(disconnect_majority_token::disconnect_majority_token),
            )
    })
    .bind(&opts.bind_address)
    .unwrap()
    .run()
    .await
    .unwrap();
}
