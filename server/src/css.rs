use actix_files::NamedFile;
use actix_web::get;

#[get("/style.css")]
pub async fn css_page() -> NamedFile {
    //include_bytes!("../style.css")
    NamedFile::open("server/style.css").unwrap()
}
