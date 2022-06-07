mod common;
pub use common::*;

pub mod extractor;

pub mod pages;

pub mod message;

mod extension;
use database::{model::MajorityToken, HackClient};
pub use extension::*;

use extractor::RequestData;
use fluent_templates::{ArcLoader, Loader, LanguageIdentifier};
use message::{MessageKind, Messages};
use pmd_hack_storage::{Query, Storage, Tag};
use url::Url;

//TODO: move to it's own file
pub struct AppData {
    // must possible be a base, otherwise, it would panic
    pub root_url: Url,
    pub storage: Storage,
    pub hack_client: HackClient,
    /// String: description of the reason
    /// Query: when does it match
    pub hidden_by_default: Vec<(String, Query)>,
    pub use_majority_token: bool,
    pub locales: ArcLoader,
}

impl AppData {
    pub fn base_url(&self, request_data: &RequestData) -> Url {
        let mut url = self.root_url.clone();
        url.query_pairs_mut()
            .append_pair("lang", &request_data.language.to_string());
        url
    }

    pub fn route_hack_file(&self, hack_slug: &str, hack_file: &str) -> Url {
        self.route_simple_static(&[hack_slug, hack_file])
    }

    pub fn route_hack(&self, request_data: &RequestData, hack_slug: &str) -> Url {
        self.route_simple(request_data, &[hack_slug])
    }

    pub fn route_hack_list_by_tag(&self, request_data: &RequestData, tag: &Tag) -> Url {
        self.route_simple(request_data, &["tagged", &tag.0])
    }

    pub fn route_index_root(&self) -> Url {
        self.route_simple_static(&["index"])
    }

    pub fn route_taginfo_file(&self) -> Url {
        self.route_simple_static(&["index", "taginfo.json"])
    }

    pub fn route_index_hacks(&self) -> Url {
        self.route_simple_static(&["index", "hacks"])
    }

    pub fn route_index_hack(&self, hack_slug: &str) -> Url {
        self.route_simple_static(&["index", "taginfo.json", hack_slug])
    }

    pub fn route_style_css(&self) -> Url {
        self.route_simple_static(&["style.css"])
    }

    pub fn route_simple(&self, request_data: &RequestData, keys: &[&str]) -> Url {
        let mut url = self.base_url(request_data);
        url.path_segments_mut().unwrap().extend(keys);
        url
    }

    pub fn route_simple_static(&self, keys: &[&str]) -> Url {
        let mut url = self.root_url.clone();
        url.path_segments_mut().unwrap().extend(keys);
        url
    }

    pub fn route_this_page(&self, request_data: &RequestData) -> Url {
        let mut url = self.base_url(request_data);
        url.set_path(&request_data.path);
        url
    }

    pub fn add_get_param_or_root_with_redirect_error(
        &self,
        url: &str,
        key: &str,
        value: &str,
    ) -> Url {
        if let Ok(mut url) = Url::parse(url) {
            url.query_pairs_mut().append_pair(key, value);
            url
        } else {
            let mut url = self.root_url.clone();
            url.query_pairs_mut()
                //TODO: display an error message when this is in the URL
                .append_pair("redirect_url_error", "true");
            url
        }
    }

    /// Check if a majority token is valid. Return None if it is valid, Some(message) otherwise, with the message being in the user's language
    /// Result : majority token is valid, a boolean indicated whether the token allow access to major only content, and a last boolean indicating whether the token can create another token
    pub async fn check_validity_of_majority_token(
        &self,
        majority_token: &str,
        messages: &mut Messages,
        lang: &LanguageIdentifier
    ) -> (Option<MajorityToken>, bool, bool) {
        match self.hack_client.get_majority_token(majority_token).await {
            Ok(Some(majority)) => {
                if majority.admin_flags.get().revoked {
                    messages.add_message_from_string(
                        self.locales.lookup(lang, "message-majority-token-invalidated-by-admin"),
                        MessageKind::Error,
                    );
                    (Some(majority), false, false)
                } else {
                    let can_certify = majority.admin_flags.get().can_certify;
                    (Some(majority), true, can_certify)
                }
            }
            Ok(None) => {
                messages.add_message_from_string(
                    self.locales.lookup(lang, "message-majority-token-does-not-exist"),
                    MessageKind::Error,
                );
                (None, false, false)
            }
            Err(e) => {
                println!(
                    "an error occured while checking the majority of the user : {:?}",
                    e
                );
                messages.add_message_from_string(
                    self.locales.lookup(lang, "message-majority-token-unexpected-error"),
                    MessageKind::Error
                );
                (None, false, false)
            }
        }
    }
}

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
