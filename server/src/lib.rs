mod common;
pub use common::*;

pub mod extractor;

pub mod pages;

pub mod message;

mod extension;
pub use extension::*;

mod app_data;
pub use app_data::AppData;

/// Return true if the hack id is illegal (due to being reserved/user for a page)
/// Currently unused
pub fn is_illegal_hack_slug(name: &str) -> bool {
    if name.contains('/') {
        true
    } else {
        matches!(
            name,
            //already user
            "Oswald-Medium.ttf"|
            "style.css"|
            "majority"|
            "tagged"|
            "create_majority_token"|
            "disconnect_majority_token"|
            "connect_majority_token"|
            "index"|

            //likely to be used
            "faq"|
            "information"|
            "search"|
            "filter"|
            "download"|
            "marius"|

            //too generic name
            "pokemon-mystery-dungeon"|
            "adventurers"|
            "explorers"|
            "super"|
            "gates"
        )
    }
}

#[cfg(test)]
mod test {
    use crate::is_illegal_hack_slug;

    #[test]
    pub fn test_illegal_hack_slug() {
        assert!(is_illegal_hack_slug("I/am"));
        assert!(is_illegal_hack_slug("faq"));
        assert!(!is_illegal_hack_slug("hello"));
    }
}
