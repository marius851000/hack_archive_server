use crate::{add_base, storage::Query, storage::QueryIssue, RequestData};
use maud::{html, Markup};
use rocket::get;

#[get("/search?<q>")]
pub fn search(data: RequestData, q: String) -> Markup {
    let query = Query::parse(&q).filter(data.get_filter());
    let (results, issues) = query.search(&data.storage);

    add_base(
        html! {
            div class="search_result_header" {
                @if !issues.is_empty() {
                    p { "Some issue happened with your query" }
                    ul {
                        @for issue in &issues {
                            li {
                                p {
                                    (match &issue {
                                        &QueryIssue::UnknownTag(tag) => format!("unknown tag \"{}\"", tag)
                                    })
                                }
                            }
                        }
                    }
                }
                p {
                    (results.len()) " result found."
                }
            }
            div class="search_result" {
                ul {
                    @for result in &results {
                        li { (result) }
                    }
                }
            }
        },
        "search page",
        &data,
    )
}
