use std::sync::Arc;

use actix_web::{get, web::Data, HttpResponse};
use database::get_timestamp;
use maud::html;

use crate::{extractor::UserData, wrap_page, AppData, PageInfo};

const TITLE: &str = "new majority token";

#[get("/create_majority_token")]
pub async fn create_majority_token(
    app_data: Data<Arc<AppData>>,
    user_data: UserData,
) -> HttpResponse {
    fn get_error_response(
        message: &str,
        discourage_reload: bool,
        app_data: &AppData,
        user_data: UserData,
    ) -> HttpResponse {
        wrap_page(
            html!(
                h1 { (TITLE) }
                p { (message) }
            ),
            PageInfo {
                name: TITLE.to_string(),
                discourage_reload,
            },
            app_data,
            user_data,
        )
    }

    if let Some(token) = &user_data.majority {
        if user_data.can_certify {
            if get_timestamp() > token.latest_certification_timestamp + 10 {
                wrap_page(
                    html!(
                        h1 { (TITLE) }
                        p { "you can certify a user. TODO" }
                    ),
                    PageInfo {
                        name: TITLE.to_string(),
                        discourage_reload: true,
                    },
                    &app_data,
                    user_data,
                )
            } else {
                get_error_response(
                    &format!(
                        "Too soon. You will be able to create a new token in {} seconds",
                        get_timestamp() - token.latest_certification_timestamp + 10
                    ),
                    true,
                    &app_data,
                    user_data,
                )
            }
        } else {
            get_error_response(
                "Your token doesn't allow to certify someone",
                false,
                &app_data,
                user_data,
            )
        }
    } else {
        get_error_response(
            "You haven't entered a majority token",
            false,
            &app_data,
            user_data,
        )
    }
}
