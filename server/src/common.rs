use crate::{extractor::RequestData, AppData};
use actix_web::{cookie::Cookie, http::StatusCode, HttpResponse};
use comrak::{markdown_to_html, ComrakOptions};
use maud::{html, Markup, PreEscaped};
use pmd_hack_storage::{Hack, Tag};

pub struct PageInfo {
    pub name: String,
    pub discourage_reload: bool,
}

pub fn wrap_page(
    markup: Markup,
    page_info: PageInfo,
    app_data: &AppData,
    request_data: RequestData,
) -> HttpResponse {
    let markup = html!(
        html {
            head {
                meta charset="utf-8" {}
                title { (page_info.name) }
                link rel="stylesheet" href=(app_data.route_static("style.css")) {}
            }
            body {
                header {
                    a id="archivedlink" href=(app_data.route(&request_data, "")) { "Archived hacks list" }
                    a id="newslink" href="https://hacknews.pmdcollab.org/" { "Return to the news site" }
                }
                main {
                    //TODO: better error message displaying. In particular, separate error from other more generic message
                    @if !request_data.messages.is_empty() {
                        div class="errorcontainer" {
                            @if request_data.messages.have_error() {
                                p {
                                    "Error occured while generating this page :"
                                }
                            }
                            @for error_message in request_data.messages.messages() {
                                div class="errormessage" {
                                    p {
                                        (error_message.value().clone())
                                    }
                                }
                            }
                        }
                    }

                    (markup)
                }
                footer {
                    p {
                        "Archive created and maintained by marius851000 ("
                        code { "marius851000#2522" }
                        " on Discord). This site is not directly affiliated, and not to be confused with the "
                        a href="https://hacks.skytemple.org" { span class="skytemple" {"SkyTemple"} " hack list" } "."
                    }
                    @if app_data.use_majority_token {
                        @if let Some(majority_check) = request_data.majority.as_ref() {
                            form action=(app_data.route(&request_data, "disconnect_majority_token")) method="post" {
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
                                input type="hidden" id="redirect_url" name="redirect_url" value=(app_data.route(&request_data, &request_data.path)) {}
                                input type="hidden" id="disconnect_majority_token" name="disconnect_majority_token" value="true" {}
                                input type="submit" value="Disconnect" {}
                            }
                            @if request_data.can_certify {
                                p {
                                    "You can create a token for another user on the "
                                    a href=(app_data.route(&request_data, "majority")) { "information page" }
                                    "."
                                }
                            }
                        }
                        form {
                            @if page_info.discourage_reload {
                                p { "Go to a non-interactive page to enter a majority token or disconnect it (it would reload the page)."}
                            } @else {
                                label for="majority_token" {
                                    "Majority code ("
                                    a href=(app_data.route(&request_data, "majority")) { "more info" }
                                    ") "
                                }
                                //TODO: use a form
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

    if let Some(token) = request_data.majority_cookie_to_set.as_ref() {
        response_builder.cookie(Cookie::build("majority_token", token).finish());
    };

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
                    a href=(app_data.route_hack(request_data, hack_id)) {
                        (hack.data.name)
                    }
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
        a href=(app_data.route_hack_list_by_tag(request_data, tag)) {
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
                    1 => ", and",
                    2.. => ", ",
                    _ => ""
                }
            }
        }
    }
}
