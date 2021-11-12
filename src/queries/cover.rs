use super::{Distrib, Selector, SelectorResult};
use crate::{data::caniuse::CANIUSE_LITE_USAGE, error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use std::ops::ControlFlow;

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^cover\s+(\d*\.?\d+)%$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct CoverSelector;

impl Selector for CoverSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult<'a> {
        if let Some(cap) = REGEX.captures(text) {
            let coverage = cap[1].parse().map_err(Error::ParsePercentage)?;
            let result = CANIUSE_LITE_USAGE.iter().try_fold(
                (vec![], 0.0f32),
                |(mut versions, total), (name, version, usage)| {
                    if total >= coverage || *usage == 0.0 {
                        ControlFlow::Break((versions, total))
                    } else {
                        versions.push(Distrib::new(name, version));
                        ControlFlow::Continue((versions, total + usage))
                    }
                },
            );
            let versions = match result {
                ControlFlow::Break((versions, _)) => versions,
                _ => unreachable!(),
            };
            Ok(Some(versions))
        } else {
            Ok(None)
        }
    }
}
