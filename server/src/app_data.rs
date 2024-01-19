use arc_swap::ArcSwap;
use database::{model::MajorityToken, HackClient};
use fluent_templates::{ArcLoader, LanguageIdentifier};
use pmd_hack_storage::{Query, Storage, Tag};
use url::Url;

use crate::{
    extractor::RequestData,
    message::{MessageKind, Messages},
    FluentLookupInfaillable,
};

pub struct AppData {
    // must possible be a base, otherwise, it would panic
    pub root_url: Url,
    pub storage: ArcSwap<Storage>,
    pub hack_client: HackClient,
    /// String: description of the reason
    /// Query: when does it match
    pub hidden_by_default: Vec<(String, Query)>,
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

    pub fn route_hack_decompress_file_list(
        &self,
        request_data: &RequestData,
        hack_slug: &str,
        hack_file: &str,
    ) -> Url {
        self.route_simple(request_data, &["decompress", hack_slug, hack_file, ""])
    }

    pub fn route_hack_decompress_file(&self, hack_slug: &str, hack_file: &str, path: &str) -> Url {
        self.route_simple_static(&["decompress", hack_slug, hack_file, path])
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
        lang: &LanguageIdentifier,
    ) -> (Option<MajorityToken>, bool, bool) {
        match self.hack_client.get_majority_token(majority_token).await {
            Ok(Some(majority)) => {
                if majority.admin_flags.get().revoked {
                    messages.add_message_from_string(
                        self.locales.lookup_infaillable(
                            lang,
                            "message-majority-token-invalidated-by-admin",
                        ),
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
                    self.locales
                        .lookup_infaillable(lang, "message-majority-token-does-not-exist"),
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
                    self.locales
                        .lookup_infaillable(lang, "message-majority-token-unexpected-error"),
                    MessageKind::Error,
                );
                (None, false, false)
            }
        }
    }
}
