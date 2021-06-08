#![feature(proc_macro_hygiene, decl_macro)]

pub mod storage;

pub mod view;

mod html;
pub use html::{add_base, SiteData};

mod request_data;
pub use request_data::RequestData;
