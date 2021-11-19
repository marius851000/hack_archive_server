use std::{fs::File, io::Write, path::PathBuf};

use maud::{html, Markup};
use pmd_hack_archive_server::storage::Hack;
use pmd_hack_archive_server::storage::Query;
use pmd_hack_archive_server::storage::Storage;
use pmd_hack_archive_server::storage::Tag;

const BASE_URL: &str = "https://hacknews.pmdcollab.org/archive/hacks";

fn main() {
    let hacks_folder = PathBuf::from("/home/marius/pmd_hack_archive/hacks");
    let suggestive_tag = Tag("suggestive".into());
    let explicit_tag = Tag("explicit".into());

    let mut storage = Storage::default();
    storage.load_all_hacks_from_folder(&hacks_folder).unwrap();

    let normal_hacks = (Query::Difference(
        Box::new(Query::All),
        Box::new(Query::AtLeastOneOfTag(vec![
            suggestive_tag.clone(),
            explicit_tag.clone(),
        ])),
    ))
    .get_matching(&storage)
    .0;
    let suggestive_hacks = (Query::AtLeastOneOfTag(vec![suggestive_tag]))
        .get_matching(&storage)
        .0;
    let explicit_hacks = (Query::AtLeastOneOfTag(vec![explicit_tag]))
        .get_matching(&storage)
        .0;

    // create the main page
    let main_page = wrap_page(
        html!(
            h1 { "Marius's archive of PMD hack-rom" }
            p {
                "This is the part of my archive that store rom-hacks patches. "
                "The goal of this archive is to save every version of every hacks. "
            }
            p {
                "If you see there is an hack or a version that is missing, don't hesitate to contact me on Discord at marius851000#2522 (or any other one)."
            }
            h2 { "List of hacks" }
            (make_hack_list(&normal_hacks))
            @if !suggestive_hacks.is_empty() {
                details {
                    summary {
                        "Hack marked as suggestive"
                    }
                    (make_hack_list(&suggestive_hacks))
                }
            }
            @if !explicit_hacks.is_empty() {
                details {
                    summary {
                        "Hack marked as explicit"
                    }
                    (make_hack_list(&explicit_hacks))
                }
            }
        ),
        PageInfo {
            name: "Archive of PMD hacks".into(),
        },
    );

    let mut index_file = File::create(hacks_folder.join("index.html")).unwrap();
    index_file
        .write_all(main_page.into_string().as_bytes())
        .unwrap();

    for (hack_id, hack) in &storage.hacks {
        hack.check_files();
        
        let hack_page = wrap_page(
            html!(
                h1 { (hack.data.name) }

                @if let Some(description) = &hack.data.description {
                    p { i { 
                        (description)
                    }}
                }
                
                @if !hack.data.authors.is_empty() {
                    p { "made by :" }
                    ul {
                        @for author in &hack.data.authors {
                            li { (author) }
                        }
                    }
                }

                @if !hack.data.tags.is_empty() {
                    p { "tags :" }
                    ul {
                        @for tag in &hack.data.tags {
                            li { (tag) }
                        }
                    }
                }

                @if let Some(source) = &hack.data.source {
                    p { "source : " a href=(source) { (source) }}
                }

                @if let Some(skytemple_db_id) = &hack.data.skytemple_db_id {
                    a href=(format!("https://hacks.skytemple.org/h/{}", skytemple_db_id)) {
                        (format!("See on the SkyTemple hack list (id {})", skytemple_db_id))
                    }
                }

                @if !hack.data.screenshots.is_empty() {
                    p { "screenshots" }
                    ul {
                        @for screenshot in &hack.data.screenshots {
                            li {
                                img src=(format!("./{}", screenshot)) { }
                            }
                        }
                    }
                }

                @if !hack.data.links.is_empty() {
                    p { "external links" }
                    ul {
                        @for (name, url) in &hack.data.links {
                            li {
                                a href=(url) { (name) }
                            }
                        }
                    }
                }

                h2 { "files" }
                @if hack.data.files.is_empty() {
                    p { "no files" }
                } else {
                    @for file in &hack.data.files {
                        hr {}
                        h3 { (file.label) }
                        p {
                            a href=(format!("{}/{}/{}", BASE_URL, hack_id, file.filename)) { "download" }
                        }
                        @if let Some(description) = &file.description {
                            i { (description) }
                        }
                        @if let Some(base) = &file.base {
                            //TODO: human-compatible name (create a new json file to store this ?)
                            p { (format!("base rom : {}", base)) }
                        }
                        @if !file.language.is_empty() {
                            p { "languages" }
                            ul {
                                @for language in &file.language {
                                    li { (language) }
                                }
                            }
                        }
                    }
                    hr {}
                }


            ),
            PageInfo {
                name: format!("Archive of {}", hack.data.name),
            },
        );
        let mut hack_index_page = File::create(hack.folder.join("index.html")).unwrap();
        hack_index_page
            .write_all(hack_page.into_string().as_bytes())
            .unwrap();
    }
}
