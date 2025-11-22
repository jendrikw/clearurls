// Rustc lints
#![forbid(unsafe_code)]
#![warn(elided_lifetimes_in_associated_constant)]
#![warn(elided_lifetimes_in_paths)]
#![warn(future_incompatible)]
#![warn(keyword_idents)]
#![warn(let_underscore)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(non_ascii_idents)]
#![warn(nonstandard_style)]
#![warn(noop_method_call)]
#![warn(refining_impl_trait)]
#![warn(rust_2018_compatibility)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(rust_2024_compatibility)]
#![warn(unused)]
#![warn(unused_extern_crates)]
#![warn(unused_import_braces)]
// Clippy categories
#![warn(clippy::cargo)]
#![warn(clippy::complexity)]
#![warn(clippy::correctness)]
#![warn(clippy::nursery)]
#![warn(clippy::pedantic)]
#![warn(clippy::perf)]
#![warn(clippy::style)]
#![warn(clippy::suspicious)]
// selected clippy lints from nursery and restriction
#![allow(clippy::redundant_pub_crate)] // I like it my way
#![warn(clippy::cognitive_complexity)]
#![warn(clippy::dbg_macro)]
#![warn(clippy::debug_assert_with_mut_call)]
#![warn(clippy::empty_line_after_outer_attr)]
#![warn(clippy::empty_structs_with_brackets)]
#![warn(clippy::float_cmp_const)]
#![warn(clippy::float_equality_without_abs)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::option_if_let_else)]
#![warn(clippy::print_stderr)]
#![warn(clippy::print_stdout)]
#![warn(clippy::suspicious_operation_groupings)]
#![warn(clippy::unseparated_literal_suffix)]
#![warn(clippy::use_debug)]
#![warn(clippy::useless_let_if_seq)]
#![warn(clippy::wildcard_dependencies)]
// Rustdoc lints
#![warn(rustdoc::broken_intra_doc_links)]
#![warn(rustdoc::missing_crate_level_docs)]

#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#![allow(clippy::doc_markdown)]
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
//! let res = cleaner.clear_single_url_str("https://example.com/test?utm_source=abc")?;
//! assert_eq!(res, "https://example.com/test");
//! # Ok(())
//! # }
//! ```

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
extern "C" {}

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

use alloc::borrow::Cow;
use core::fmt::{Display, Formatter};
use core::str::{FromStr, Utf8Error};
use regex::Regex;
use url::{ParseError, Url};

use rules::Rules;

mod deserialize_utils;
mod rules;
#[cfg(test)]
#[allow(clippy::mod_module_files)]
mod tests;

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

    /// Construct a [`UrlCleaner`] with rules from a [reader][std::io::Read], most often a [`File`][std::fs::File]
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
    #[allow(clippy::missing_const_for_fn)]
    pub fn strip_referral_marketing(mut self, value: bool) -> Self {
        self.strip_referral_marketing = value;
        self
    }

    /// Clean a single URL.
    ///
    /// The argument is a string that is *just* a URL, with no text around.
    ///
    /// The Cleaning may involve
    /// - 1. removing tracking parameters
    ///      and/or,
    /// - 2. detecting redirections with the target url in a query parameters
    ///
    /// # Returns
    /// a cleaned URL
    ///
    /// # Errors
    /// If an error occurred. See the [`Error`] enum for possible reasons.
    pub fn clear_single_url_str<'a>(&self, url: &'a str) -> Result<Cow<'a, str>, Error> {
        if url.starts_with("data:") {
            return Ok(Cow::Borrowed(url));
        }
        let mut result = Url::from_str(url)?;
        for p in &self.rules.providers {
            if p.match_url(result.as_str()) {
                result = p.remove_fields_from_url(&result, self.strip_referral_marketing)?;
            }
        }

        Ok(Cow::Owned(result.into()))
    }

    /// Clean a single URL.
    ///
    /// The Cleaning may involve
    /// - 1. removing tracking parameters
    ///      and/or,
    /// - 2. detecting redirections with the target url in a query parameters
    ///
    /// # Returns
    /// a cleaned URL
    ///
    /// # Errors
    /// If an error occurred. See the [`Error`] enum for possible reasons.
    pub fn clear_single_url<'a>(&self, url: &'a Url) -> Result<Cow<'a, Url>, Error> {
        if url.scheme().starts_with("data") {
            return Ok(Cow::Borrowed(url));
        }
        let mut url = Cow::Borrowed(url);
        for p in &self.rules.providers {
            if p.match_url(url.as_str()) {
                url = Cow::Owned(p.remove_fields_from_url(&url, self.strip_referral_marketing)?);
            }
        }

        Ok(url)
    }

    /// Clean all URLs in a text.
    ///
    /// This may involve
    /// - 1. removing tracking parameters
    ///      and/or,
    /// - 2. detecting redirections with the target url in a query parameters
    ///
    /// # Returns
    /// The string with all URLs inside cleaned.
    /// Text outside of URLs is left unchanged.
    ///
    /// # Errors
    /// Alls errors encountered are returned in a [`Vec`][alloc::vec::Vec].
    #[cfg(feature = "linkify")]
    pub fn clear_text<'a>(&self, s: &'a str) -> Result<Cow<'a, str>, alloc::vec::Vec<Error>> {
        self.clear_text_with_linkfinder(s, &linkify::LinkFinder::new())
    }


    /// Clean all URLs in a text.
    ///
    /// This may involve
    /// - 1. removing tracking parameters
    ///      and/or,
    /// - 2. detecting redirections with the target url in a query parameters
    ///
    /// # Returns
    /// The string with all URLs inside cleaned.
    /// Text outside of URLs is left unchanged.
    ///
    /// # Errors
    /// Alls errors encountered are returned in a [`Vec`][alloc::vec::Vec].
    #[cfg(feature = "linkify")]
    pub fn clear_text_with_linkfinder<'a>(
        &self,
        s: &'a str,
        finder: &linkify::LinkFinder,
    ) -> Result<Cow<'a, str>, alloc::vec::Vec<Error>> {
        use alloc::vec::Vec;
        use alloc::string::String;

        let mut spans = Vec::new();
        let mut errors = Vec::new();

        for res in finder.spans(s) {
            match res.kind() {
                Some(linkify::LinkKind::Url) => match self.clear_single_url_str(res.as_str()) {
                    Ok(cow) => spans.push(cow),
                    Err(e) => errors.push(e),
                },
                _ => spans.push(Cow::Borrowed(res.as_str())),
            }
        }

        if errors.is_empty() {
            if spans.iter().all(|s| matches!(s, Cow::Borrowed(_))) {
                Ok(Cow::Borrowed(s))
            } else {
                Ok(Cow::Owned(spans.into_iter().collect::<String>()))
            }
        } else {
            Err(errors)
        }
    }

    /// Clean all URLs in a Markdown document. This affects all kinds of URLs, like
    /// - proper Markdown Links
    /// - auto links (links inside angle brackets)
    /// - links to images
    /// - bare links with no extra markup.
    ///
    /// The document will be modified in-place.
    ///
    /// # Errors
    /// The algorithm continues with the rest of the document if an error occurs.
    /// The return value is `Ok(())` if there were no errors.
    /// Otherwise, the list of errors is returned as the `Err` value.
    #[cfg(feature = "markdown-it")]
    pub fn clear_markdown(&self, doc: &mut markdown_it::Node) -> Result<(), alloc::vec::Vec<Error>> {
        use markdown_it::parser::inline::Text;
        use markdown_it::plugins::cmark::inline::autolink::Autolink;
        use markdown_it::plugins::cmark::inline::image::Image;
        use markdown_it::plugins::cmark::inline::link::Link;
        use markdown_it::plugins::extra::linkify::Linkified;
        use markdown_it::Node;
        use alloc::string::String;

        fn replace_url(cleaner: &UrlCleaner, url: &mut String) -> Result<(), Error> {
            match cleaner.clear_single_url_str(url)? {
                Cow::Borrowed(_) => {}
                Cow::Owned(new_url) => {
                    *url = new_url;
                }
            }
            Ok(())
        }

        fn callback(cleaner: &UrlCleaner, node: &mut Node) -> Result<(), Error> {
            if let Some(link) = node.cast_mut::<Autolink>() {
                replace_url(cleaner, &mut link.url)?;
                node.children = alloc::vec![Node::new(Text {
                    content: link.url.clone()
                })];
            }
            if let Some(link) = node.cast_mut::<Linkified>() {
                replace_url(cleaner, &mut link.url)?;
                node.children = alloc::vec![Node::new(Text {
                    content: link.url.clone()
                })];
            }
            if let Some(link) = node.cast_mut::<Link>() {
                replace_url(cleaner, &mut link.url)?;
            }
            if let Some(link) = node.cast_mut::<Image>() {
                replace_url(cleaner, &mut link.url)?;
            }
            Ok(())
        }

        let mut result = alloc::vec![];
        doc.walk_mut(|node, _| {
            if let Err(e) = callback(self, node) {
                result.push(e);
            }
        });

        if result.is_empty() {
            Ok(())
        } else {
            Err(result)
        }
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
            Self::FileRead(x) => write!(f, "error reading rules: {x}"),
            Self::RuleSyntax(x) => write!(f, "error parsing rules: {x}"),
            Self::UrlSyntax(x) => write!(f, "error parsing url: {x}"),
            Self::RedirectionHasNoCapturingGroup(x) => {
                write!(f, "redirection regex {x} has no capture group")
            }
            Self::PercentDecodeUtf8Error(x) => {
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
            Self::FileRead(e) => Some(e),
            Self::RuleSyntax(e) => Some(e),
            Self::UrlSyntax(e) => Some(e),
            Self::RedirectionHasNoCapturingGroup(_) => None,
            Self::PercentDecodeUtf8Error(e) => Some(e),
        }
    }
}
