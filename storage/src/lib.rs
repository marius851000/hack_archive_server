mod storage;
pub use storage::{Storage, StorageLoadError};

mod hack;
pub use hack::{Hack, HackData, HackLoadError};

mod tags;
pub use tags::{Tag, Tags};
pub const MAJORONLY_CATEGORY: &str = "majoronly";

mod query;
pub use query::{Query, QueryIssue};

mod taginfo;
pub use taginfo::{TagInfo, TagInfoLoadError};
