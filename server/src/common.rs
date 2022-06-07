use std::collections::HashMap;

use crate::{extractor::RequestData, message::MessageKind, AppData};
use actix_web::{cookie::Cookie, http::StatusCode, HttpResponse};
use comrak::{markdown_to_html, ComrakOptions};
use maud::{html, Markup, PreEscaped};
use pmd_hack_storage::{Hack, Query, Tag};

pub struct PageInfo {
    pub name: String,
    pub discourage_reload: bool,
    pub display_majority_info: bool,
}

pub fn wrap_page(
    markup: Markup,
    page_info: PageInfo,
    app_data: &AppData,
    request_data: RequestData,
) -> HttpResponse {
    let mut credit_args = HashMap::new();
    credit_args.insert(
        "skytemple_hack_link_start",
        "<a href=\"https://hacks.skytemple.org\">".into(),
    );
    credit_args.insert("skytemple_hack_link_end", "</a>".into());
    let markup = html!(
        html {
            head {
                meta charset="utf-8" {}
                title { (page_info.name) }
                link rel="stylesheet" href=(app_data.route_style_css().as_str()) {}
            }
            body {
                header {
                    a id="archivedlink" href=(app_data.base_url(&request_data).as_str()) { (request_data.lookup("return-to-main-page-link")) }
                    a id="newslink" href="https://hacknews.pmdcollab.org/" { (request_data.lookup("return-to-news-site-link")) }
                }
                main {
                    //TODO: better error message displaying. In particular, separate error from other more generic message
                    @if !request_data.messages.is_empty() {
                        div class="messagecontainer" {
                            @for error_message in request_data.messages.messages() {
                                div class=(if error_message.kind() == MessageKind::Error { "message errormessage" } else { "message successmessage" }) {
                                    p {
                                        (error_message.value())
                                    }
                                }
                            }
                        }
                    }

                    (markup)
                }
                footer {
                    p {
                        (PreEscaped(request_data.lookup_with_args("footer-credit", &credit_args)))
                    }
                    @if app_data.use_majority_token && (page_info.display_majority_info || request_data.majority_token.is_some()) {
                        @if let Some(majority_check) = request_data.majority_token.as_ref() {
                            form action=(app_data.route_simple(&request_data, &["disconnect_majority_token"]).as_str()) method="post" {
                                label for="disconnect_majority_token" {
                                    @if request_data.have_access_to_major_only_content {
                                        (format!("You are connected with the valid majority token {}. ", majority_check._id))
                                    } @else {
                                        "You are connected with the "
                                        b { "invalid" }
                                        " majority token "
                                        (majority_check._id)
                                        "."
                                    }
                                }
                                input type="hidden" id="redirect_url" name="redirect_url" value=(app_data.route_this_page(&request_data).as_str()) {}
                                input type="submit" value="Disconnect" {}
                            }
                            @if request_data.can_certify {
                                p {
                                    "You can create a token for another user on the "
                                    a href=(app_data.route_simple(&request_data, &["majority"]).as_str()) { "information page" }
                                    "."
                                }
                            }
                        }
                        @if page_info.discourage_reload {
                            p { "Go to a non-interactive page to enter a majority token or disconnect it (it would reload the page)."}
                        } @else {
                            form action=(app_data.route_simple(&request_data, &["connect_majority_token"]).as_str()) method="post" {
                                label for="majority_token" {
                                    "Majority code ("
                                    a href=(app_data.route_simple(&request_data, &["majority"]).as_str()) { "more info" }
                                    ") "
                                }
                                input type="hidden" id="redirect_url" name="redirect_url" value=(app_data.route_this_page(&request_data).as_str()) {}
                                input type="text" id="majority_token" name="majority_token" {}
                                input type="submit" value="Submit" {}
                            }
                        }
                    }
                    p {
                        "Site data can be mirrored with rclone using the http directory at "
                        a href="https://hacknews.pmdcollab.org/archive" { "hacknews.pmdcollab.org/archive" }
                        "."
                    }
                    p {
                        "Source code of the site avalaible on " a href="https://github.com/marius851000/hack_archive_server" { "GitHub" } "."
                    }
                }
            }
        }
    );

    let markup: String = markup.into();
    let mut response_builder = HttpResponse::build(StatusCode::OK);
    response_builder.content_type(mime::TEXT_HTML_UTF_8);
    response_builder.cookie(Cookie::build("messages", "").finish());

    response_builder.body(markup.into_boxed_str().into_string())
}

pub fn make_hack_list(
    hacks: &[(String, &Hack)],
    request_data: &RequestData,
    app_data: &AppData,
) -> Markup {
    html! {
        ul {
            @for (hack_id, hack) in hacks {
                li {
                    a href=(app_data.route_hack(request_data, hack_id).as_str()) {
                        (hack.data.name)
                    }
                }
            }
        }
    }
}

pub fn make_hack_list_hidden(
    query: Query,
    request_data: &RequestData,
    app_data: &AppData,
) -> Markup {
    let unfiltered_hacks = (Query::Difference(
        Box::new(query.clone()),
        Box::new(Query::Or(
            app_data
                .hidden_by_default
                .iter()
                .map(|(_t, q)| q.clone())
                .collect(),
        )),
    ))
    .get_matching(&app_data.storage)
    .0;

    html! {
        (make_hack_list(&unfiltered_hacks, request_data, app_data))
        @for (hidden_string, hidden_query) in &app_data.hidden_by_default {
            @let hidden_hacks = Query::Intersection(Box::new(query.clone()), Box::new(hidden_query.clone())).get_matching(&app_data.storage).0;
            @if !hidden_hacks.is_empty() {
                details {
                    summary {
                        (hidden_string) " (" (request_data.lookup("hidden-click-to-reveal")) ")"
                    }
                    (make_hack_list(&hidden_hacks, request_data, app_data))
                }
            }
        }
    }
}

pub fn render_markdown(text: &str) -> PreEscaped<String> {
    PreEscaped(markdown_to_html(text, &ComrakOptions::default()))
}

pub fn render_tag(tag: &Tag, request_data: &RequestData, app_data: &AppData) -> Markup {
    html! {
        a href=(app_data.route_hack_list_by_tag(request_data, tag).as_str()) {
            @if let Some(single_tag_info) = app_data.storage.taginfo.get_tag(tag) {
                @let label = single_tag_info.label.as_ref().unwrap_or(&tag.0);
                @if let Some(category_data) = app_data.storage.taginfo.get_category_for_single_tag_info(single_tag_info, tag) {
                    span class="tag" style=(format!("border-color:{};background-color:{}", category_data.border_color, category_data.background_color)) { (label) }
                } @else {
                    span class="tag" { (label) }
                }
            } @else {
                span class="tag" { (tag.0) }
            }
        }
    }
}

pub fn render_many_tags(tags: Vec<Tag>, request_data: &RequestData, app_data: &AppData) -> Markup {
    let tags = app_data.storage.taginfo.orders_tags(tags);
    html! {
        p class="tagslist" {
            "tags : "
            @for (count, tag) in tags.iter().enumerate() {
                @let remaining = tags.len() - count - 1;
                (render_tag(tag, request_data, app_data));
                @match remaining {
                    1 => ", and ",
                    2.. => ", ",
                    _ => ""
                }
            }
        }
    }
}
