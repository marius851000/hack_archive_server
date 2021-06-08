#![feature(proc_macro_hygiene, decl_macro)]

use std::path::PathBuf;

use pmd_hack_archive_server::{storage::Storage, view, SiteData};

#[macro_use]
extern crate rocket;

fn main() -> anyhow::Result<()> {
    let data = SiteData {
        name: "unofficial pmd romhack database".into(),
    };

    let storage = Storage::load_from_folder(&PathBuf::from("/home/marius/pmd_hack_archive"))?;

    rocket::ignite()
        .manage(data)
        .manage(storage)
        .mount(
            "/",
            routes![
                view::view_main_page,
                view::view_change_filter,
                view::view_set_filter,
                view::search
            ],
        )
        .launch();

    Ok(())
}
