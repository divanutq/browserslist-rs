use super::{Selector, SelectorResult};
use crate::{opts::Opts, resolve};

pub(super) struct DefaultsSelector;

impl Selector for DefaultsSelector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult {
        if text.eq_ignore_ascii_case("defaults") {
            resolve(
                ["> 0.5%", "last 2 versions", "Firefox ESR", "not dead"],
                opts,
            )
            .map(Some)
        } else {
            Ok(None)
        }
    }
}
