use std::collections::HashSet;

use super::{Hack, Storage, Tag};

/// An error that happenned while performing a [`Query`]. Multiple may happend in a single query.
pub enum QueryIssue {
    /// an unknown tag is used
    UnknownTag(Tag),
    /// The hack with the given slug can't be found (inconsistencie error with the database, internal error)
    HackNotFound(String),
}

/// A query describing the hack that should returned by a search
#[derive(Clone)]
pub enum Query {
    /// At least one tag of the list should match (or)
    AtLeastOneOfTag(Vec<Tag>),
    /// At least one query should match
    Or(Vec<Query>),
    /// Match every hack
    All,
    /// The element should match the first query, but not the second
    Difference(Box<Query>, Box<Query>),
    /// The element should match both query
    Intersection(Box<Query>, Box<Query>),
}

impl Query {
    pub fn get_matching<'a>(
        &self,
        storage: &'a Storage,
    ) -> (Vec<(String, &'a Hack)>, Vec<QueryIssue>) {
        let (query_result, mut issues) = self.get_list_of_matching_hack(storage);

        let mut result = Vec::new();
        for hack_slug in query_result.iter() {
            match storage.hacks.get(hack_slug) {
                Some(hack) => result.push((hack_slug.to_string(), hack)),
                None => issues.push(QueryIssue::HackNotFound(hack_slug.to_string())),
            };
        }

        result.sort_unstable_by(|(slug1, _), (slug2, _)| slug1.cmp(slug2));

        (result, issues)
    }

    fn get_list_of_matching_hack(&self, storage: &Storage) -> (HashSet<String>, Vec<QueryIssue>) {
        match self {
            Self::AtLeastOneOfTag(tags) => {
                let mut issues = Vec::new();
                let mut result = HashSet::new();
                for tag in tags {
                    match storage.tags.get_hack_for_tag(tag) {
                        Some(v) => {
                            result.extend(v.iter().cloned());
                        }
                        None => issues.push(QueryIssue::UnknownTag(tag.clone())),
                    };
                }
                (result, issues)
            }
            Self::Or(queries) => {
                let mut issues = Vec::new();
                let mut result = HashSet::new();
                for query in queries {
                    let (r, i) = query.get_list_of_matching_hack(storage);
                    result.extend(r);
                    issues.extend(i);
                }
                (result, issues)
            }
            Query::Difference(first_query, second_query) => {
                let (first_result, mut issues) = first_query.get_list_of_matching_hack(storage);
                let (second_result, second_issues) =
                    second_query.get_list_of_matching_hack(storage);
                let result = first_result
                    .difference(&second_result)
                    .cloned()
                    .collect::<HashSet<String>>();
                issues.extend(second_issues);
                (result, issues)
            }
            Query::Intersection(query_1, query_2) => {
                let (result, mut issues) = query_1.get_list_of_matching_hack(storage);
                let (result_2, issues_2) = query_2.get_list_of_matching_hack(storage);
                let result = result.intersection(&result_2).cloned().collect();
                issues.extend(issues_2);
                (result, issues)
            }
            &Query::All => (
                storage.hacks.keys().map(|x| x.to_string()).collect(),
                Vec::new(),
            ),
        }
    }
}
