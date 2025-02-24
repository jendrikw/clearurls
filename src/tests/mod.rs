#![allow(clippy::trivial_regex)]

use super::*;
use crate::rules::Provider;
use crate::Error::{PercentDecodeUtf8Error, RedirectionHasNoCapturingGroup};
use alloc::string::ToString;
use alloc::vec;
use regex::RegexSet;
use serde_json::error::Category;
#[cfg(feature = "std")]
use std::error::Error as _;
#[cfg(feature = "std")]
use std::io::{Seek, SeekFrom, Write};

const _: () = {
    const fn assert_auto_traits<T: Send + Sync + 'static>() {}
    assert_auto_traits::<UrlCleaner>();
    assert_auto_traits::<Error>();
};

#[allow(edition_2024_expr_fragment_specifier)]
macro_rules! assert_matches {
    ($e:expr, $pat:pat $(if $guard:expr)? $(,)?) => {
        assert!(matches!($e, $pat $(if $guard)?), "assertion failed: {:?} does not match {}", $e, stringify!($pat $(if $guard)?))
    };
}

#[test]
fn test_referral_marketing_setter() {
    let cleaner = UrlCleaner::from_rules_str(r#"{"providers":{}}"#).unwrap();
    assert!(!cleaner.strip_referral_marketing);
    let cleaner = cleaner.strip_referral_marketing(true);
    assert!(cleaner.strip_referral_marketing);
}

#[test]
fn test_strip_referral_marketing() {
    let provider = Provider {
        url_pattern: Regex::new("https://example.com").unwrap(),
        rules: vec![],
        raw_rules: vec![],
        referral_marketing: vec![Regex::new("ref").unwrap()],
        exceptions: RegexSet::default(),
        redirections: vec![],
    };
    let res = provider
        .remove_fields_from_url(&Url::from_str("https://example.com?ref=1").unwrap(), true)
        .unwrap();
    assert_eq!(res.as_str(), "https://example.com/");
}

//noinspection RegExpSimplifiable
#[test]
fn test_invalid_redirection() {
    let provider = Provider {
        url_pattern: Regex::new("^https?://(?:[a-z0-9-]+\\.)*?google(?:\\.[a-z]{2,}){1,}").unwrap(),
        rules: vec![],
        raw_rules: vec![],
        referral_marketing: vec![Regex::new("ref").unwrap()],
        exceptions: RegexSet::default(),
        // this regex is missing a capturing group around the last https...
        redirections: vec![Regex::new("^https?://(?:[a-z0-9-]+\\.)*?google(?:\\.[a-z]{2,}){1,}/url\\?.*?(?:url|q)=https?[^&]+").unwrap()],
    };
    let err = provider
        .remove_fields_from_url(
            &Url::from_str("https://google.co.uk/url?foo=bar&q=http%3A%2F%2Fexample.com%2Fimage.png&bar=foo").unwrap(),
            false,
        )
        .unwrap_err();
    assert_matches!(err, RedirectionHasNoCapturingGroup(_));
    assert_eq!(err.to_string(), "redirection regex ^https?://(?:[a-z0-9-]+\\.)*?google(?:\\.[a-z]{2,}){1,}/url\\?.*?(?:url|q)=https?[^&]+ has no capture group");
    #[cfg(feature = "std")]
    {
        assert!(err.source().is_none());
    }
}

//noinspection RegExpSimplifiable
#[test]
fn test_invalid_urldecode() {
    let provider = Provider {
        url_pattern: Regex::new("^https?://(?:[a-z0-9-]+\\.)*?google(?:\\.[a-z]{2,}){1,}").unwrap(),
        rules: vec![],
        raw_rules: vec![],
        referral_marketing: vec![Regex::new("ref").unwrap()],
        exceptions: RegexSet::default(),
        redirections: vec![Regex::new("^https?://(?:[a-z0-9-]+\\.)*?google(?:\\.[a-z]{2,}){1,}/url\\?.*?(?:url|q)=(https?[^&]+)").unwrap()],
    };
    // a byte F0 is not valid utf 8
    let err = provider
        .remove_fields_from_url(&Url::from_str("https://google.co.uk/url?foo=bar&q=http%F0").unwrap(), false)
        .unwrap_err();
    assert_matches!(err, PercentDecodeUtf8Error(_));
    #[cfg(feature = "std")]
    {
        assert_matches!(err, PercentDecodeUtf8Error(ref inner) if error_ptr_eq(inner, err.source().unwrap()));
    }
    assert_eq!(
        err.to_string(),
        "percent decoding resulted in non-UTF-8 bytes: incomplete utf-8 byte sequence from index 4"
    );
}

#[test]
fn test_raw_rules_unchanged() {
    let provider = Provider {
        url_pattern: Regex::new("^https?://pantip.com").unwrap(),
        rules: vec![],
        raw_rules: vec![Regex::new("#lead.*").unwrap()],
        referral_marketing: vec![],
        exceptions: RegexSet::default(),
        redirections: vec![],
    };
    let res = provider.remove_fields_from_url(&Url::from_str("https://pantip.com/").unwrap(), false);
    assert_eq!(res.unwrap().as_str(), "https://pantip.com/");
}

#[test]
fn test_raw_rules_produce_invalid_url() {
    let provider = Provider {
        url_pattern: Regex::new("https://example.com").unwrap(),
        rules: vec![],
        raw_rules: vec![Regex::new("https://").unwrap()],
        referral_marketing: vec![],
        exceptions: RegexSet::default(),
        redirections: vec![],
    };
    let err = provider
        .remove_fields_from_url(&Url::from_str("https://example.com").unwrap(), false)
        .unwrap_err();
    assert_matches!(err, Error::UrlSyntax(_));
    #[cfg(feature = "std")]
    {
        assert_matches!(err, Error::UrlSyntax(ref inner) if error_ptr_eq(inner, err.source().unwrap()));
    }
}

#[test]
#[cfg(feature = "std")]
fn test_from_read_vec() {
    let data = br#"{"providers":{"example":{"urlPattern":"","rules":["foo"]}}}"#;
    let c = UrlCleaner::from_rules_file(&data[..]).unwrap();
    assert_eq!(c.rules.providers.len(), 1);
    assert_eq!(c.rules.providers[0].rules.len(), 1);
    assert_eq!(c.rules.providers[0].rules[0].as_str(), "foo");
}

#[test]
#[cfg(feature = "std")]
fn test_from_file_invalid_json() {
    let err = UrlCleaner::from_rules_file(b"[".as_slice()).unwrap_err();
    assert_matches!(err, Error::RuleSyntax(ref e) if e.classify() == Category::Eof);
}

#[test]
fn test_from_str_invalid_json() {
    let err = UrlCleaner::from_rules_str("[").unwrap_err();
    assert_matches!(err, Error::RuleSyntax(ref e) if e.classify() == Category::Eof);
    #[cfg(feature = "std")]
    {
        assert_matches!(err, Error::RuleSyntax(ref inner) if error_ptr_eq(inner, err.source().unwrap()));
    }
    assert_eq!(
        err.to_string(),
        "error parsing rules: EOF while parsing a list at line 1 column 1"
    );
}

#[test]
#[cfg(feature = "std")]
fn test_from_read_file() {
    let mut file = tempfile::tempfile().unwrap();
    file.write_all(br#"{"providers":{"example":{"urlPattern":"","rules":["foo"]}}}"#)
        .unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    let c = UrlCleaner::from_rules_file(&file).unwrap();
    assert_eq!(c.rules.providers.len(), 1);
    assert_eq!(c.rules.providers[0].rules.len(), 1);
    assert_eq!(c.rules.providers[0].rules[0].as_str(), "foo");
}

#[test]
#[cfg(feature = "std")]
fn test_from_path() {
    let mut file = tempfile::NamedTempFile::new().unwrap();
    file.write_all(br#"{"providers":{"example":{"urlPattern":"","rules":["foo"]}}}"#)
        .unwrap();
    file.seek(SeekFrom::Start(0)).unwrap();
    let c = UrlCleaner::from_rules_path(file.path()).unwrap();
    assert_eq!(c.rules.providers.len(), 1);
    assert_eq!(c.rules.providers[0].rules.len(), 1);
    assert_eq!(c.rules.providers[0].rules[0].as_str(), "foo");
}

#[test]
#[cfg(feature = "std")]
fn test_from_invalid_path() {
    let file = tempfile::NamedTempFile::new().unwrap();
    let path = file.path().to_path_buf();
    file.close().unwrap();
    let err = UrlCleaner::from_rules_path(&path).unwrap_err();
    assert_matches!(err, Error::FileRead(ref e) if e.kind() == std::io::ErrorKind::NotFound);
    assert_matches!(err, Error::FileRead(ref inner) if error_ptr_eq(inner, err.source().unwrap()));
    assert!(err.to_string().starts_with("error reading rules: "));
}

#[test]
fn test_remove_fields_from_url_errors() {
    let provider = UrlCleaner {
        rules: Rules {
            providers: vec![Provider {
                url_pattern: Regex::new(".*").unwrap(),
                rules: vec![],
                raw_rules: vec![],
                referral_marketing: vec![],
                exceptions: RegexSet::default(),
                redirections: vec![],
            }],
        },
        strip_referral_marketing: false,
    };
    let err = provider.clear_single_url_str("//example.com").unwrap_err();
    assert_matches!(err, Error::UrlSyntax(_));
    #[cfg(feature = "std")]
    {
        assert_matches!(err, Error::UrlSyntax(ref inner) if error_ptr_eq(inner, err.source().unwrap()));
    }
    assert_eq!(
        err.to_string(),
        "error parsing url: relative URL without a base"
    );
}

#[cfg(feature = "std")]
fn error_ptr_eq<T: std::error::Error + 'static>(
    x: &T,
    y: &(dyn std::error::Error + 'static),
) -> bool {
    #[allow(clippy::option_if_let_else)]
    match y.downcast_ref::<T>() {
        Some(y) => core::ptr::eq(x, y),
        None => false,
    }
}
