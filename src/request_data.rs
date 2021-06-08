use rocket::{
    http::{Cookie, Cookies},
    request::FromRequest,
    Outcome, State,
};

use crate::{
    storage::{Filter, Storage},
    SiteData,
};

pub struct RequestData<'a> {
    pub cookies: Cookies<'a>,
    pub storage: State<'a, Storage>,
    pub site_data: State<'a, SiteData>,
    filter: Filter,
}

impl<'a, 'r> FromRequest<'a, 'r> for RequestData<'a> {
    type Error = ();

    fn from_request(
        request: &'a rocket::Request<'r>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        let storage = match request.guard::<State<Storage>>() {
            Outcome::Failure(e) => return Outcome::Failure(e),
            Outcome::Success(s) => s,
            Outcome::Forward(f) => return Outcome::Forward(f),
        };
        let site_data = match request.guard::<State<SiteData>>() {
            Outcome::Failure(e) => return Outcome::Failure(e),
            Outcome::Success(s) => s,
            Outcome::Forward(f) => return Outcome::Forward(f),
        };
        let mut cookies = match request.guard::<Cookies>() {
            Outcome::Failure(_) => panic!(),
            Outcome::Success(s) => s,
            Outcome::Forward(f) => return Outcome::Forward(f),
        };

        let filter = match cookies.get("filter") {
            Some(filter_cookie) => storage
                .filters
                .get(filter_cookie.value())
                .unwrap_or(storage.filters.get_default()),
            None => {
                cookies.add(Cookie::new("filter", storage.filters.default.to_string()));
                storage.filters.get_default()
            }
        };

        Outcome::Success(Self {
            cookies,
            storage,
            site_data,
            filter,
        })
    }
}

impl<'a> RequestData<'a> {
    pub fn get_filter(&self) -> &Filter {
        &self.filter
    }

    pub fn set_filter(&mut self, filter_slug: &str) -> bool {
        match self.storage.filters.get(filter_slug) {
            Some(_) => {
                self.cookies.add(
                    Cookie::build("filter", filter_slug.to_string())
                        .path("/")
                        .finish(),
                );
                true
            }
            None => false,
        }
    }
}
