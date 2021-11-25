#![allow(clippy::float_cmp)]
#![deny(clippy::if_not_else)]
#![deny(clippy::needless_borrow)]
#![deny(clippy::trivial_regex)]
#![deny(clippy::unimplemented)]
#![warn(missing_docs)]

//! **browserslist-rs** is a Rust-based implementation of [Browserslist](https://github.com/browserslist/browserslist).
//!
//! ## Introduction
//!
//! This library bundles Can I Use data, Electron versions list and Node.js releases list,
//! so it won't and doesn't need to access any data files.
//!
//! Except several non-widely/non-frequently used features,
//! this library works as same as the JavaScript-based
//! implementation [Browserslist](https://github.com/browserslist/browserslist).
//!
//! ## Usage
//!
//! It provides a simple API for querying which accepts a sequence of strings and options [`Opts`],
//! then returns the result.
//!
//! ```
//! use browserslist::{Distrib, Opts, resolve, Error};
//!
//! let distribs = resolve(["ie <= 6"], &Opts::new()).unwrap();
//! assert_eq!(distribs[0].name(), "ie");
//! assert_eq!(distribs[0].version(), "6");
//! assert_eq!(distribs[1].name(), "ie");
//! assert_eq!(distribs[1].version(), "5.5");
//!
//! assert_eq!(
//!     resolve(["yuru 1.0"], &Opts::new()),
//!     Err(Error::BrowserNotFound(String::from("yuru")))
//! );
//! ```
//!
//! The result isn't a list of strings, instead, it's a tuple struct called [`Distrib`].
//! If you need to retrieve something like JavaScript-based implementation of
//! [Browserslist](https://github.com/browserslist/browserslist),
//! you can convert them to strings:
//!
//! ```
//! use browserslist::{Distrib, Opts, resolve, Error};
//!
//! let distribs = resolve(["ie <= 6"], &Opts::new()).unwrap();
//! assert_eq!(
//!     distribs.into_iter().map(|d| d.to_string()).collect::<Vec<_>>(),
//!     vec![String::from("ie 6"), String::from("ie 5.5")]
//! );
//! ```
//!
//! ## WebAssembly
//!
//! This crate can be compiled as WebAssembly, without configuring any features manually.
//!
//! Please note that browser and Deno can run WebAssembly,
//! but those environments aren't Node.js,
//! so you will receive an error when querying `current node` in those environments.
//!
//! ## N-API
//!
//! If you're going to use this crate in a Node.js addon via N-API directly or indirectly,
//! we've supported for it. You just need to enable the `node` feature.
//!
//! If you're targeting Node.js, we recommend you to use N-API over WebAssembly,
//! because it's faster and less-limited than the WebAssembly-build.

use parser::{parse, Query};
use std::cmp::Ordering;
#[cfg(target_arch = "wasm32")]
pub use wasm::execute;
pub use {error::Error, opts::Opts, queries::Distrib};

mod config;
mod data;
mod error;
#[cfg(feature = "node")]
mod node;
mod opts;
mod parser;
mod queries;
mod semver;
#[cfg(test)]
mod test;
#[cfg(target_arch = "wasm32")]
mod wasm;

/// Resolve browserslist queries.
///
/// This is a low-level API.
/// If you want to load queries from configuration file and
/// resolve them automatically,
/// use the higher-level API [`execute`] instead.
///
/// ```
/// use browserslist::{Distrib, Opts, resolve};
///
/// let distribs = resolve(["ie <= 6"], &Opts::new()).unwrap();
/// assert_eq!(distribs[0].name(), "ie");
/// assert_eq!(distribs[0].version(), "6");
/// assert_eq!(distribs[1].name(), "ie");
/// assert_eq!(distribs[1].version(), "5.5");
/// ```
pub fn resolve<I, S>(queries: I, opts: &Opts) -> Result<Vec<Distrib>, Error>
where
    S: AsRef<str>,
    I: IntoIterator<Item = S>,
{
    let mut distribs = vec![];

    for query in queries {
        parse(query.as_ref()).try_fold(&mut distribs, |distribs, current| {
            let query_string = match current {
                Query::And(s) => s,
                Query::Or(s) => s,
            };

            let is_exclude = query_string.starts_with("not");
            let query_string = if is_exclude {
                &query_string[4..]
            } else {
                query_string
            };

            let mut queries = queries::query(query_string, opts)?;
            if is_exclude {
                distribs.retain(|q| !queries.contains(q));
            } else {
                match current {
                    Query::And(_) => {
                        distribs.retain(|q| queries.contains(q));
                    }
                    Query::Or(_) => {
                        distribs.append(&mut queries);
                    }
                }
            }

            Ok::<_, Error>(distribs)
        })?;
    }

    distribs.sort_by(|a, b| match a.name().cmp(b.name()) {
        Ordering::Equal => {
            let version_a = a.version().split('-').next().unwrap();
            let version_b = b.version().split('-').next().unwrap();
            version_b
                .parse::<semver::Version>()
                .unwrap_or_default()
                .cmp(&version_a.parse().unwrap_or_default())
        }
        ord => ord,
    });
    distribs.dedup();

    Ok(distribs)
}

/// Load queries from configuration with environment information,
/// then resolve those queries.
///
/// If you want to resolve custom queries (not from configuration file),
/// use the lower-level API [`resolve`] instead.
///
/// ```
/// use browserslist::{Opts, execute};
///
/// assert!(execute(&Opts::new()).unwrap().is_empty());
/// ```
#[cfg(not(target_arch = "wasm32"))]
pub fn execute(opts: &Opts) -> Result<Vec<Distrib>, Error> {
    resolve(config::load(opts)?, opts)
}
