pub mod storage;

mod html;
pub use html::{add_base, SiteData};

mod request_data;
pub use request_data::RequestData;