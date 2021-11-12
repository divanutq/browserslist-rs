use super::{Distrib, Selector, SelectorResult};
use crate::{data::caniuse::CANIUSE_LITE_USAGE, error::Error, opts::Opts};
use once_cell::sync::Lazy;
use regex::Regex;

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([<>]=?)\s*(\d*\.?\d+)%$").unwrap());

pub(super) struct PercentageSelector;

impl Selector for PercentageSelector {
    fn select<'a>(&self, text: &'a str, _: &Opts) -> SelectorResult {
        let cap = match REGEX.captures(text) {
            Some(cap) => cap,
            None => return Ok(None),
        };

        let sign = &cap[1];
        let popularity: f32 = cap[2].parse().map_err(Error::ParsePercentage)?;

        let versions = CANIUSE_LITE_USAGE
            .iter()
            .filter(|(_, _, usage)| match sign {
                ">" => *usage > popularity,
                "<" => *usage < popularity,
                "<=" => *usage <= popularity,
                _ => *usage >= popularity,
            })
            .map(|(name, version, _)| Distrib::new(name, version))
            .collect();
        Ok(Some(versions))
    }
}
