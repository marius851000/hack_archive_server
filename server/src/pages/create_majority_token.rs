use std::collections::BTreeSet;

use actix_web::{get, web::Data, HttpResponse};
use database::{
    get_timestamp,
    model::{MajorityToken, MajorityTokenAdminFlags},
    FieldWithTime, HackClient, HackClientError,
};
use maud::html;
use rand::{distributions::Alphanumeric, Rng};

use crate::{extractor::RequestData, wrap_page, AppData, PageInfo};

const TITLE: &str = "new majority token";

#[get("/create_majority_token")]
pub async fn create_majority_token(
    app_data: Data<AppData>,
    hack_client: Data<HackClient>,
    mut request_data: RequestData,
) -> HttpResponse {
    fn get_error_response(
        message: &str,
        discourage_reload: bool,
        app_data: &AppData,
        request_data: RequestData,
    ) -> HttpResponse {
        wrap_page(
            html!(
                h1 { (TITLE) }
                p { (message) }
            ),
            PageInfo {
                name: TITLE.to_string(),
                discourage_reload,
                display_majority_info: true,
            },
            app_data,
            request_data,
        )
    }

    fn get_hack_client_error_response_and_log_error(
        error: HackClientError,
        app_data: &AppData,
        request_data: RequestData,
    ) -> HttpResponse {
        //TODO: pretty print of error
        log::error!(
            "An error occured communicating with the database while adding a new token : {:?}",
            error
        );
        get_error_response(
            "An error occured while communicating with the database. Something is wrong with this server.",
            false,
            app_data,
            request_data
        )
    }

    if let Some(majority_token) = &mut request_data.majority {
        if request_data.can_certify {
            if get_timestamp() > majority_token.latest_certification_timestamp + 10 {
                let mut remaining_max_loop: u8 = 100;
                let new_token_id = loop {
                    remaining_max_loop = remaining_max_loop.saturating_sub(1);
                    if remaining_max_loop == 0 {
                        log::error!("Not able to generate a majority token in less than 100 iteration. Something is certainly wrong !");
                        return get_error_response(
                            "Unable to generate a unused token. Somethin on the server is certainly horrible wrong",
                            false,
                            &app_data, request_data);
                    };
                    let token: String = rand::thread_rng()
                        .sample_iter(&Alphanumeric)
                        .take(16)
                        .map(char::from)
                        .collect();
                    match hack_client.get_majority_token(&token).await {
                        Ok(Some(_)) => continue,
                        Ok(None) => break token,
                        Err(e) => {
                            return get_hack_client_error_response_and_log_error(
                                e,
                                &app_data,
                                request_data,
                            )
                        }
                    };
                };

                // 1. add the token to the list of certified token by the current user
                majority_token.certify.insert(new_token_id.clone());
                majority_token.latest_certification_timestamp = get_timestamp();
                match hack_client
                    .save_majority_token(majority_token.clone())
                    .await
                {
                    Ok(_) => (),
                    Err(e) => {
                        return get_hack_client_error_response_and_log_error(
                            e,
                            &app_data,
                            request_data,
                        )
                    }
                };
                // 2. Create the certified
                let new_token = MajorityToken {
                    _id: new_token_id.clone(),
                    _rev: String::new(),
                    _deleted: None,
                    certify: BTreeSet::new(),
                    admin_flags: FieldWithTime::new(MajorityTokenAdminFlags {
                        can_certify: true,
                        need_certification: true,
                        revoked: false,
                    }),
                    latest_certification_timestamp: 0,
                    _conflicts: Vec::new(),
                };
                match hack_client.save_majority_token(new_token).await {
                    Ok(_) => (),
                    Err(e) => {
                        return get_hack_client_error_response_and_log_error(
                            e,
                            &app_data,
                            request_data,
                        )
                    }
                }

                log::info!(
                    "new majority token created by {} : {}",
                    majority_token._id,
                    new_token_id
                );

                wrap_page(
                    html!(
                        h1 { (TITLE) }
                        p { "New majority token created. Share it with the user you certified have more than 18 years, and keep using your own token." }

                        p { "The other person will be able to use it to accept hacks restricting to major user." }
                        p { "The token is "
                            b { (new_token_id) }
                        }
                    ),
                    PageInfo {
                        name: TITLE.to_string(),
                        discourage_reload: true,
                        display_majority_info: true,
                    },
                    &app_data,
                    request_data,
                )
            } else {
                get_error_response(
                    &format!(
                        "Too soon. You will be able to create a new token in {} seconds",
                        majority_token
                            .latest_certification_timestamp
                            .saturating_add(10)
                            .saturating_sub(get_timestamp())
                    ),
                    true,
                    &app_data,
                    request_data,
                )
            }
        } else {
            get_error_response(
                "Your token doesn't allow to certify someone",
                false,
                &app_data,
                request_data,
            )
        }
    } else {
        get_error_response(
            "You haven't entered a majority token",
            false,
            &app_data,
            request_data,
        )
    }
}
