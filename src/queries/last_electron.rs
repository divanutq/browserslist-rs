use super::{Selector, SelectorResult, Version};
use crate::{data::electron::ELECTRON_VERSIONS, error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^last\s+(\d+)\s+electron\s+versions?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct LastElectronSelector;

impl Selector for LastElectronSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult<'a> {
        if let Some(cap) = REGEX.captures(text) {
            let count: usize = cap[1].parse().map_err(Error::ParseVersionsCount)?;
            let versions = ELECTRON_VERSIONS
                .iter()
                .rev()
                .take(count)
                .map(|(_, version)| Version::new("chrome", &version))
                .collect();
            Ok(Some(versions))
        } else {
            Ok(None)
        }
    }
}
