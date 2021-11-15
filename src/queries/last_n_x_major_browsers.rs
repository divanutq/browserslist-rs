use super::{count_android_filter, should_filter_android, Distrib, Selector, SelectorResult};
use crate::{data::caniuse::get_browser_stat, error::Error, opts::Opts};
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^last\s+(\d+)\s+(\w+)\s+major\s+versions?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct LastNXMajorBrowsersSelector;

impl Selector for LastNXMajorBrowsersSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        let cap = match REGEX.captures(text) {
            Some(cap) => cap,
            None => return Ok(None),
        };
        let count = cap[1].parse().map_err(Error::ParseVersionsCount)?;
        let name = &cap[2];

        let (name, stat) = get_browser_stat(name, opts.mobile_to_desktop)
            .ok_or_else(|| Error::BrowserNotFound(name.to_string()))?;
        let count = if should_filter_android(name, opts.mobile_to_desktop) {
            count_android_filter(count, opts.mobile_to_desktop)
        } else {
            count
        };
        let minimum = stat
            .released
            .iter()
            .rev()
            .map(|version| version.split('.').next().unwrap())
            .dedup()
            .nth(count - 1)
            .and_then(|minimum| minimum.parse().ok())
            .unwrap_or(0);

        let versions = stat
            .released
            .iter()
            .filter(move |version| {
                version.split('.').next().unwrap().parse().unwrap_or(0) >= minimum
            })
            .rev()
            .map(move |version| Distrib::new(name, version))
            .collect();

        Ok(Some(versions))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::run_compare;
    use test_case::test_case;

    #[test_case("last 2 edge major versions"; "basic")]
    #[test_case("last 1 bb major version"; "support pluralization")]
    #[test_case("last 3 Chrome major versions"; "case insensitive")]
    #[test_case("last 2 android major versions"; "android")]
    #[test_case("last 2 bb major versions"; "non-sequential version numbers")]
    #[test_case("last 3 bb major versions"; "more versions than have been released")]
    fn default_options(query: &str) {
        run_compare(query, &Opts::new());
    }

    #[test]
    fn mobile_to_desktop() {
        run_compare(
            "last 2 android major versions",
            Opts::new().mobile_to_desktop(true),
        );
    }
}
