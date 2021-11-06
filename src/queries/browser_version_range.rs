use super::Selector;
use crate::{
    data::caniuse::{get_browser_stat, CANIUSE_LITE_VERSION_ALIASES},
    opts::Opts,
};
use once_cell::sync::Lazy;
use regex::Regex;

static REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^(\w+)\s*(>=?|<=?)\s*([\d.]+)$").unwrap());

pub(super) struct BrowserVersionRangeSelector;

impl Selector for BrowserVersionRangeSelector {
    fn select(&self, text: &str, opts: &Opts) -> Option<Vec<String>> {
        let matches = REGEX.captures(text)?;
        let name = matches.get(1)?.as_str();
        let sign = matches.get(2)?.as_str();
        let version = matches.get(3)?.as_str();

        let stat = get_browser_stat(name, opts.mobile_to_desktop)?;
        let version: f32 = match CANIUSE_LITE_VERSION_ALIASES.get(&stat.name)?.get(version) {
            Some(version) => version.parse().unwrap_or(0.0),
            None => version.parse().unwrap_or(0.0),
        };

        let versions = stat
            .released
            .iter()
            .map(|v| v.parse::<f32>().unwrap_or(0.0))
            .filter(|v| match sign {
                ">" => *v > version,
                "<" => *v < version,
                "<=" => *v <= version,
                _ => *v >= version,
            })
            .map(|version| format!("{} {}", &stat.name, version))
            .collect();
        Some(versions)
    }
}
