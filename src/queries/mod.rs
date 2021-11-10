use crate::{data::caniuse, error::Error, opts::Opts};
use std::fmt::Display;

mod browser_bounded_range;
mod browser_unbounded_range;
mod cover;
mod dead;
mod defaults;
mod electron_accurate;
mod electron_bounded_range;
mod electron_unbounded_range;
mod firefox_esr;
mod last_n_browsers;
mod last_n_electron;
mod last_n_electron_major;
mod last_n_major_browsers;
mod last_n_x_browsers;
mod last_n_x_major_browsers;
mod node_accurate;
mod node_bounded_range;
mod node_unbounded_range;
mod op_mini;
mod percentage;
mod phantom;
mod since;
mod unreleased_browsers;
mod unreleased_electron;
mod unreleased_x_browsers;
mod years;

/// Representation of browser name (or `node`) and its version.
///
/// When converting it to string, it will be formatted as the output of
/// [browserslist](https://github.com/browserslist/browserslist). For example:
///
/// ```
/// use browserslist::Distrib;
///
/// assert_eq!(Distrib::new("firefox", "93").to_string(), "firefox 93".to_string());
/// assert_eq!(Distrib::new("op_mini", "all").to_string(), "op_mini all".to_string());
/// assert_eq!(Distrib::new("node", "16.0.0").to_string(), "node 16.0.0".to_string());
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct Distrib<'a>(&'a str, &'a str);

impl<'a> Distrib<'a> {
    #[inline]
    pub fn new(name: &'a str, version: &'a str) -> Self {
        Self(name, version)
    }

    /// Return browser name, or `node`.
    ///
    /// ```
    /// use browserslist::Distrib;
    ///
    /// assert_eq!(Distrib::new("firefox", "93").name(), "firefox");
    /// assert_eq!(Distrib::new("op_mini", "all").name(), "op_mini");
    /// assert_eq!(Distrib::new("node", "16.0.0").name(), "node");
    /// ```
    #[inline]
    pub fn name(&self) -> &str {
        self.0
    }

    /// Return version string.
    ///
    /// ```
    /// use browserslist::Distrib;
    ///
    /// assert_eq!(Distrib::new("firefox", "93").version(), "93");
    /// assert_eq!(Distrib::new("op_mini", "all").version(), "all");
    /// assert_eq!(Distrib::new("node", "16.0.0").version(), "16.0.0");
    /// ```
    #[inline]
    pub fn version(&self) -> &str {
        self.1
    }
}

impl<'a> Display for Distrib<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.0, self.1)
    }
}

pub type SelectorResult<'a> = Result<Option<Vec<Distrib<'a>>>, Error>;

trait Selector {
    fn select<'a>(&self, text: &'a str, opts: &Opts) -> SelectorResult<'a>;
}

pub fn query<'a>(query_string: &'a str, opts: &Opts) -> Result<Vec<Distrib<'a>>, Error> {
    let selectors: Vec<Box<dyn Selector>> = vec![
        Box::new(last_n_major_browsers::LastNMajorBrowsersSelector),
        Box::new(last_n_browsers::LastNBrowsersSelector),
        Box::new(last_n_electron_major::LastNElectronMajorSelector),
        Box::new(last_n_x_major_browsers::LastNXMajorBrowsersSelector),
        Box::new(last_n_electron::LastNElectronSelector),
        Box::new(last_n_x_browsers::LastNXBrowsersSelector),
        Box::new(unreleased_browsers::UnreleasedBrowsersSelector),
        Box::new(unreleased_electron::UnreleasedElectronSelector),
        Box::new(unreleased_x_browsers::UnreleasedXBrowsersSelector),
        Box::new(years::YearsSelector),
        Box::new(since::SinceSelector),
        Box::new(percentage::PercentageSelector),
        Box::new(cover::CoverSelector),
        Box::new(electron_bounded_range::ElectronBoundedRangeSelector),
        Box::new(node_bounded_range::NodeBoundedRangeSelector),
        Box::new(browser_bounded_range::BrowserBoundedRangeSelector),
        Box::new(electron_unbounded_range::ElectronUnboundedRangeSelector),
        Box::new(node_unbounded_range::NodeUnboundedRangeSelector),
        Box::new(browser_unbounded_range::BrowserUnboundedRangeSelector),
        Box::new(firefox_esr::FirefoxESRSelector),
        Box::new(op_mini::OperaMiniSelector),
        Box::new(electron_accurate::ElectronAccurateSelector),
        Box::new(node_accurate::NodeAccurateSelector),
        Box::new(phantom::PhantomSelector),
        Box::new(defaults::DefaultsSelector),
        Box::new(dead::DeadSelector),
    ];

    for selector in selectors {
        if let Some(distribs) = selector.select(query_string, opts)? {
            return Ok(distribs);
        }
    }
    Ok(vec![])
}

#[inline]
pub fn should_filter_android(name: &str, mobile_to_desktop: bool) -> bool {
    name == "android" && !mobile_to_desktop
}

pub fn count_android_filter(count: usize, mobile_to_desktop: bool) -> usize {
    let released = &caniuse::get_browser_stat("android", mobile_to_desktop)
        .unwrap()
        .released;
    let diff = (released.last().unwrap().parse::<f32>().unwrap()
        - caniuse::ANDROID_EVERGREEN_FIRST
        - (count as f32)) as usize;
    if diff > 0 {
        1
    } else {
        1 - diff
    }
}
