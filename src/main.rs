#![feature(proc_macro_hygiene, decl_macro)]

use std::{path::PathBuf};

use maud::{Markup, html};
use pmd_hack_archive_server::{RequestData, SiteData, add_base, storage::Storage};

#[macro_use] extern crate rocket;



#[get("/")]
fn index(data: RequestData) -> Markup {
    add_base(html! {
        h1 { "Main page" }
    }, "main page", &data)
}

fn main() -> anyhow::Result<()> {
    let data = SiteData {
        name: "unofficial pmd romhack database".into(),
    };

    let storage = Storage::load_from_folder(&PathBuf::from("/home/marius/pmd_hack_archive"))?;
    
    rocket::ignite()
        .manage(data)
        .manage(storage)
        .mount("/", routes![index]).launch();

    Ok(())
}
