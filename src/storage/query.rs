use std::collections::HashSet;

use super::{Filter, Storage, Tag};

/// An error that happenned while performing a [`Query`]. Multiple may happend in a single query.
pub enum QueryIssue {
    /// an unknown tag is used
    UnknownTag(Tag),
}

/// A query describing the hack that should returned by a search
pub enum Query {
    /// At least one tag of the list should match (or)
    AtLeastOneOfTag(Vec<Tag>),
    /// The element should not match the first query, but not the second
    Difference(Box<Query>, Box<Query>),
    /// The element should match both query
    Intersection(Box<Query>, Box<Query>),
}

impl Query {
    pub fn dont_filter(self) -> QueryFiltered {
        QueryFiltered(self)
    }

    pub fn parse(query: &str) -> Query {
        let mut tag_list = Vec::new();
        for extracted_section in query.split(',') {
            tag_list.push(Tag(extracted_section.trim().to_string()))
        }
        Query::AtLeastOneOfTag(tag_list)
    }

    pub fn filter(self, filter: &Filter) -> QueryFiltered {
        QueryFiltered(Query::Difference(
            Box::new(self),
            Box::new(Query::AtLeastOneOfTag(filter.hide.clone())),
        ))
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
                let (mut result, mut issues) = query_1.get_list_of_matching_hack(storage);
                let (result_2, issues_2) = query_2.get_list_of_matching_hack(storage);
                result.extend(result_2.iter().cloned());
                issues.extend(issues_2);
                (result, issues)
            }
        }
    }
}

/// Represent a query on which censor filter where applied (creating a new, more restrictive query)
pub struct QueryFiltered(pub Query);

impl QueryFiltered {
    pub fn search(&self, storage: &Storage) -> (HashSet<String>, Vec<QueryIssue>) {
        self.0.get_list_of_matching_hack(storage)
    }

    pub fn filter(self, filter: &Filter) -> Self {
        self.0.filter(filter)
    }
}
