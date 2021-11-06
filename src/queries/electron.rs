use super::Selector;
use crate::{data::electron::ELECTRON_VERSIONS, opts::Opts};
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};

static REGEX: Lazy<Regex> = Lazy::new(|| {
    RegexBuilder::new(r"^electron\s+(\d+\.\d+)(?:\.\d+)?\s*-\s*(\d+\.\d+)(?:\.\d+)?$")
        .case_insensitive(true)
        .build()
        .unwrap()
});

pub(super) struct ElectronSelector;

impl Selector for ElectronSelector {
    fn select(&self, text: &str, _: &Opts) -> Option<Vec<String>> {
        REGEX
            .captures(text)
            .and_then(|cap| {
                Some((
                    cap.get(1)?.as_str().parse::<f32>().ok()?,
                    cap.get(2)?.as_str().parse::<f32>().ok()?,
                ))
            })
            .map(|(from, to)| {
                ELECTRON_VERSIONS
                    .iter()
                    .filter(|(version, _)| from <= *version && *version <= to)
                    .map(|(_, version)| format!("chrome {}", version))
                    .collect()
            })
    }
}
