use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use std::cmp::Ordering;

mod queries;

static REGEX_OR: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"\s+or\s+|\s*,\s*")
        .case_insensitive(true)
        .build()
        .unwrap()
});

static REGEX_AND: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"\s+and\s+")
        .case_insensitive(true)
        .build()
        .unwrap()
});

#[derive(Debug)]
enum Query<'a> {
    And(&'a str),
    Or(&'a str),
}

fn parse(query: &str) -> impl Iterator<Item = Query<'_>> {
    REGEX_OR
        .split(query)
        .map(|s| {
            REGEX_AND.split(s).enumerate().map(|(i, text)| {
                if i == 0 {
                    Query::Or(text)
                } else {
                    Query::And(text)
                }
            })
        })
        .flatten()
}

fn semver_compare(a: &str, b: &str) -> Ordering {
    a.split('.')
        .zip(b.split('.'))
        .fold(Ordering::Equal, |ord, (a, b)| {
            if ord == Ordering::Equal {
                // this is intentional: version comes from high to low
                b.parse::<i32>()
                    .unwrap_or(0)
                    .cmp(&a.parse::<i32>().unwrap_or(0))
            } else {
                ord
            }
        })
}

pub fn resolve(queries: &[impl AsRef<str>]) -> Vec<String> {
    let mut result = queries
        .iter()
        .map(|query| parse(query.as_ref()))
        .flatten()
        .fold(vec![], |mut result, current| {
            match current {
                Query::And(query_string) => {
                    let is_exclude = query_string.starts_with("not");
                    let query_string = if is_exclude {
                        &query_string[4..]
                    } else {
                        query_string
                    };
                    if let Some(queries) = queries::query(query_string) {
                        if is_exclude {
                            result.retain(|q| !queries.contains(q));
                        } else {
                            result.retain(|q| queries.contains(q));
                        }
                    }
                }
                Query::Or(query_string) => {
                    let is_exclude = query_string.starts_with("not");
                    let query_string = if is_exclude {
                        &query_string[4..]
                    } else {
                        query_string
                    };
                    if let Some(mut queries) = queries::query(query_string) {
                        if is_exclude {
                            result.retain(|q| !queries.contains(q));
                        } else {
                            result.append(&mut queries);
                        }
                    }
                }
            }
            result
        });

    result.dedup();
    result.sort_by(|a, b| {
        let mut a = a.split(' ');
        let mut b = b.split(' ');
        let browser_a = a.next().unwrap();
        let browser_b = b.next().unwrap();

        if browser_a == browser_b {
            let version_a = a.next().unwrap().split('-').next().unwrap();
            let version_b = b.next().unwrap().split('-').next().unwrap();
            semver_compare(version_a, version_b)
        } else {
            browser_a.cmp(browser_b)
        }
    });

    result
}
