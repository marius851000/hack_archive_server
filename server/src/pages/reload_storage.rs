use std::sync::Arc;

use actix_web::{error::ErrorForbidden, get, web::Data, Error, HttpResponse};
use display_error_chain::DisplayErrorChain;
use maud::html;
use pmd_hack_storage::Storage;

use crate::{extractor::RequestData, wrap_page, AppData, PageInfo};

#[get("/reload")]
pub async fn reload(
    app_data: Data<AppData>,
    request_data: RequestData,
) -> Result<HttpResponse, Error> {
    if request_data.reload_secret == Some(app_data.secrets.reload_page_password.to_string()) {
        let new_storage = Storage::load_from_folder(&app_data.archive_folder);

        let error_reporting_status = if new_storage.errors.is_empty() {
            app_data.storage.store(Arc::new(new_storage));
            html!(p { (request_data.lookup("reload-no-error")) })
        } else {
            let (error_display, status_text, should_reload) = if new_storage
                .errors
                .iter()
                .any(|e| !e.is_not_much_important())
            {
                (
                    html!(
                        h2 { (request_data.lookup("reload-error-section")) }
                        ul {
                            @for error in new_storage.errors.iter().filter(|e| !e.is_not_much_important()) {
                                li { p {
                                    (DisplayErrorChain::new(error).to_string())
                                }}
                            }
                        }
                    ),
                    html!(p { (request_data.lookup("reload-important-error-found-no-reload"))}),
                    false,
                )
            } else {
                (
                    html!(),
                    html!(p { (request_data.lookup("reload-warning-found-reload")) } ),
                    true,
                )
            };

            let warning_display = if new_storage.errors.iter().any(|e| e.is_not_much_important()) {
                html!(
                    h2 { (request_data.lookup("reload-warning-section")) }
                    ul {
                        @for warning in new_storage.errors.iter().filter(|e| e.is_not_much_important()) {
                            li { p {
                                (DisplayErrorChain::new(warning).to_string())
                            }}
                        }
                    }
                )
            } else {
                html!()
            };

            if should_reload {
                app_data.storage.store(Arc::new(new_storage));
            }

            html! {
                (status_text)
                (error_display)
                (warning_display)
            }
        };

        Ok(wrap_page(
            error_reporting_status,
            PageInfo {
                name: request_data.lookup("reload-header"),
                discourage_reload: false,
                display_majority_info: true,
            },
            &app_data,
            request_data,
        ))
    } else {
        Err(ErrorForbidden("No secret provided or invalid"))
    }
}
