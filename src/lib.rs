#![warn(future_incompatible)]
#![warn(keyword_idents)]
#![warn(let_underscore)]
#![warn(nonstandard_style)]
#![warn(refining_impl_trait)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(rust_2024_compatibility)]
#![warn(unused)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(clippy::pedantic)]
#![warn(clippy::style)]
#![warn(clippy::correctness)]
#![warn(clippy::suspicious)]
#![warn(clippy::complexity)]
#![warn(clippy::perf)]
#![warn(clippy::cargo)]
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::missing_crate_level_docs)]
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

//! This crate provides a solution to remove tracking parameters and other nuisance from URLs.
//!
//! In order to detect such parameters, this crates uses crowdsourced *Rules* from the
//! [ClearURLs browser extension](https://clearurls.xyz/).
//!
//! A set of rules is included in this library, but you can supply your own. Refer to the
//! [ClearURLs documentation](https://docs.clearurls.xyz/1.26.1/specs/rules/) for specific syntax and semantics.
//!
//! # Example
//! ```
//! # use clearurls::UrlCleaner;
//! # fn main() -> Result<(), clearurls::Error> {
//! let cleaner = UrlCleaner::from_embedded_rules()?;
//! let res = cleaner.clear_url("https://example.com/test?utm_source=abc")?;
//! assert_eq!(res, "https://example.com/test");
//! # Ok(())
//! # }
//! ```

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
extern {}

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use alloc::borrow::Cow;
use core::fmt::{Display, Formatter};
use core::str::Utf8Error;
use regex::Regex;
use url::ParseError;

use rules::Rules;

mod deserialize_utils;
mod rules;

/// A [`UrlCleaner`] can remove tracking parameters from URLs.
///
/// This struct is relatively expensive to construct because it needs to parse the rules from JSON.
/// It's recommended to create one per application and reuse it.
#[derive(Debug)]
pub struct UrlCleaner {
    rules: Rules,
    strip_referral_marketing: bool,
}

impl UrlCleaner {
    /// Construct a [`UrlCleaner`] with rules from a path, which will be opened and read.
    /// # Errors
    /// See [`Error`]
    #[cfg(feature = "std")]
    pub fn from_rules_path(path: &std::path::Path) -> Result<Self, Error> {
        Self::from_rules_file(std::fs::File::open(path)?)
    }

    /// Construct a [`UrlCleaner`] with rules from a [reader][std::io::Read], most often a [`File`]
    /// # Errors
    /// See [`Error`]
    #[cfg(feature = "std")]
    pub fn from_rules_file<R: std::io::Read>(reader: R) -> Result<Self, Error> {
        let buf = std::io::BufReader::new(reader);
        Ok(Self {
            rules: serde_json::from_reader(buf)?,
            strip_referral_marketing: false,
        })
    }

    /// # Errors
    /// See [`Error`]
    pub fn from_rules_str(rules: &str) -> Result<Self, Error> {
        Ok(Self {
            rules: serde_json::from_str(rules)?,
            strip_referral_marketing: false,
        })
    }

    /// Construct using the JSON embedded in this library.
    /// This may be outdated, but should provide a good baseline.
    ///
    /// # Errors
    /// See [`Error`]
    pub fn from_embedded_rules() -> Result<Self, Error> {
        Self::from_rules_str(include_str!("../data.minify.json"))
    }

    /// Configure whether you want to strip referral codes and similar parameters.
    ///
    /// While they can be considered to be tracking, they are useful on occasion.
    /// The default is `false`, meaning these are kept.
    #[must_use]
    pub fn strip_referral_marketing(mut self, value: bool) -> Self {
        self.strip_referral_marketing = value;
        self
    }

    /// Clean a URL. This may involve
    /// - 1. removing tracking parameters
    ///      and/or,
    /// - 2. detecting redirections with the target url in a query parameters
    ///
    /// # Returns
    /// a cleaned URL
    ///
    /// # Errors
    /// If an error occurred. See the [`Error`] enum for possible reasons.
    pub fn clear_url<'a>(&self, url: &'a str) -> Result<Cow<'a, str>, Error> {
        if url.starts_with("data:") {
            return Ok(Cow::Borrowed(url));
        }
        let mut result = Cow::Borrowed(url);
        for p in &self.rules.providers {
            if p.match_url(&result) {
                let cleaned = p.remove_fields_from_url(&result, self.strip_referral_marketing)?;
                // TODO get rid of the allocation
                result = Cow::Owned(cleaned.into_owned());
            }
        }

        Ok(result)
    }
}

/// Various errors that can happen while cleaning a URL
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An Error occurred while opening or reading a file
    #[cfg(feature = "std")]
    FileRead(std::io::Error),
    /// The provided rules is invalid json or doesn't have the expected format
    RuleSyntax(serde_json::Error),
    /// A URL could not be parsed from the input.
    UrlSyntax(ParseError),
    /// The rules contained a redirection regex that doesn't specify the target
    RedirectionHasNoCapturingGroup(Regex),
    /// Bytes that are invalid UTF-8
    PercentDecodeUtf8Error(Utf8Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            #[cfg(feature = "std")]
            Error::FileRead(x) => write!(f, "error reading rules: {x}"),
            Error::RuleSyntax(x) => write!(f, "error parsing rules: {x}"),
            Error::UrlSyntax(x) => write!(f, "error parsing url: {x}"),
            Error::RedirectionHasNoCapturingGroup(x) => {
                write!(f, "redirection regex {x} has no capture group")
            }
            Error::PercentDecodeUtf8Error(x) => {
                write!(f, "percent decoding resulted in non-UTF-8 bytes: {x}")
            }
        }
    }
}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::FileRead(value)
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::RuleSyntax(value)
    }
}

impl From<ParseError> for Error {
    fn from(value: ParseError) -> Self {
        Self::UrlSyntax(value)
    }
}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self::PercentDecodeUtf8Error(value)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::FileRead(e) => Some(e),
            Error::RuleSyntax(e) => Some(e),
            Error::UrlSyntax(e) => Some(e),
            Error::RedirectionHasNoCapturingGroup(_) => None,
            Error::PercentDecodeUtf8Error(e) => Some(e)
        }
    }
}


const _: () = {
    const fn assert_auto_traits<T: Send + Sync>() {}
    assert_auto_traits::<UrlCleaner>();
    assert_auto_traits::<Error>();
};
