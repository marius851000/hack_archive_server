//use actix_files::NamedFile;
use actix_web::get;

#[get("/style.css")]
pub async fn css() -> &'static [u8] {
    include_bytes!("../../style.css")
    //NamedFile::open("server/style.css").unwrap()
}
