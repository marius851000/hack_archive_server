use std::sync::Arc;

use actix_web::{get, http::StatusCode, web::Data, HttpResponse};
use maud::html;

use crate::{extractor::UserData, wrap_page, AppData, PageInfo};

#[get("/majority")]
pub async fn majority(app_data: Data<Arc<AppData>>, user_data: UserData) -> HttpResponse {
    if !app_data.use_majority_token {
        return HttpResponse::new(StatusCode::NOT_IMPLEMENTED);
    }
    wrap_page(
        html!(
            h1 { "Information about the majority check" }
            p {
                "This website contain some content that enter in the definition given by the French penal code 227-14 (just pornographic content, but that will change when the need happen) ("
                a href="https://www.legifrance.gouv.fr/codes/article_lc/LEGIARTI000044394218" { "read in french" }
                "). The law also include \"message with a violent character, incitation to terrorism, pornographic, including those involving one or many animals, the hability to deeply infringe human dignity or inciting minor to commit physically dangerous games".

                "These content shouldn't be able to be seen by minor. That is why I included a (rather) complicated method to check the majority of the user (note that it only check the majority of the user with a good enought precision, not the maturity of them)."

                br {}

                "Also, in France, major person are those who are 18 or more. Even if you live in Mali or other country where the majority is sooner than 18 years old. It will be 18 for everyone (also apply for country where the majority is more than 18 years old)"

                br {}

                "I have decided to use the following method, allowing a minor bit of decentralization."

                ul {
                    li {
                        "A user have a unique and personal code allowing they can enter that certify they are major."
                    }
                    li {
                        "This code can be delivered by any person that already have a code (of course, they should check themselves the person is major the way they want)."
                    }
                    li {
                        "I (marius) can also deliver these code. I will just do a quick online check to see if the user is plausibly not a minor (Yep, that's what I consider a sufficient check. But if I go deeper, I'll have to protect personal data. And that's insanely complicated and far more dangerous)."
                    }

                }

                "And... that's it !"

                br {}

                "If you have a code, you can enter it in the footer. Otherwise, feel free to contact the admin (a.k.a Marius) for a code !"

                br {}

                i { "Also, French laws outlaw child pornography (including representation of this. I'm happy JCATQFTUO didn't did that), so I won't archive these hacks." }

                br {}

                i { "Oh ! And if you know a better solution, I'll be happy to know it too !" }
            }

            @if user_data.majority.is_some() {
                hr {}
                p { "You currently have a majority token loaded. More information about it can be found at the bottom of the page." }
                @if user_data.can_certify {
                    p { "You can create for another user. You should make sure, in your own way, that they are actually major as in the legal definition in France (more than 18 year). They will themselves be able to create other majority tokens." }

                    a href=(format!("{}/create_majority_token", app_data.root_url)) { "Create a new majority token" }
                }
            }
        ),
        PageInfo {
            name: "Info about the majority check".to_string(),
            discourage_reload: false,
        },
        &app_data,
        user_data,
    )
}
