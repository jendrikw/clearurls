// SPDX-FileCopyrightText: 2026 jendrikw
//
// SPDX-License-Identifier: LGPL-3.0-only

use crate::deserialize_utils::{
    deserialize_map_as_vec, deserialize_regex, deserialize_regex_set, deserialize_regex_vec,
};
use crate::urlencoding::{parse_query_parameters, repeatedly_urldecode, search_params_to_string};
use crate::Error;
use alloc::borrow::Cow;
use alloc::str::FromStr;
use alloc::vec::Vec;
use regex::{Regex, RegexSet};
use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub(crate) struct Rules {
    #[serde(deserialize_with = "deserialize_map_as_vec")]
    pub(crate) providers: Vec<Provider>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Provider {
    #[serde(deserialize_with = "deserialize_regex")]
    pub(crate) url_pattern: Regex,
    #[serde(default, deserialize_with = "deserialize_regex_vec")]
    pub(crate) rules: Vec<regex::bytes::Regex>,
    #[serde(default, deserialize_with = "deserialize_regex_vec")]
    pub(crate) raw_rules: Vec<Regex>,
    #[serde(default, deserialize_with = "deserialize_regex_vec")]
    pub(crate) referral_marketing: Vec<regex::bytes::Regex>,
    #[serde(default, deserialize_with = "deserialize_regex_set")]
    pub(crate) exceptions: RegexSet,
    #[serde(default, deserialize_with = "deserialize_regex_vec")]
    pub(crate) redirections: Vec<Regex>,
}

impl Provider {
    pub(crate) fn remove_fields_from_url(
        &self,
        url: &Url,
        strip_referral_marketing: bool,
    ) -> Result<Url, Error> {
        if let Some(redirect) = self.get_redirection(url.as_str())? {
            let url = repeatedly_urldecode(redirect);
            return Ok(Url::from_str(&url)?);
        }
        let mut url = Cow::Borrowed(url.as_str());
        for r in &self.raw_rules {
            match r.replace_all(&url, "") {
                Cow::Borrowed(_) => {}
                Cow::Owned(new) => url = Cow::Owned(new),
            }
        }

        let mut url = Url::parse(&url)?;

        let query = url.query().unwrap_or("");
        let mut query = parse_query_parameters(query.as_bytes());

        let fragments = url.fragment().unwrap_or("");
        let mut fragments = parse_query_parameters(fragments.as_bytes());

        for r in self.get_rules(strip_referral_marketing) {
            query.retain(|(k, _)| !is_full_match(r, k));
            fragments.retain(|(k, _)| !is_full_match(r, k));
        }

        let query = search_params_to_string(&query);
        let fragments = search_params_to_string(&fragments);

        url.set_query(query.as_deref());
        url.set_fragment(fragments.as_deref());

        Ok(url)
    }

    pub(crate) fn match_url(&self, url: &str) -> bool {
        self.url_pattern.is_match(url) && !self.match_exception(url)
    }

    fn match_exception(&self, url: &str) -> bool {
        url == "javascript:void(0)" || self.exceptions.is_match(url)
    }

    fn get_redirection<'a>(&self, url: &'a str) -> Result<Option<&'a str>, Error> {
        for r in &self.redirections {
            if let Some(c) = r.captures(url) {
                let c = c
                    .get(1)
                    .ok_or_else(|| Error::RedirectionHasNoCapturingGroup(r.clone()))?;
                let s = c.as_str();
                return Ok(Some(s));
            }
        }
        Ok(None)
    }

    fn get_rules(
        &self,
        strip_referral_marketing: bool,
    ) -> impl Iterator<Item = &regex::bytes::Regex> {
        if strip_referral_marketing {
            self.rules.iter().chain(self.referral_marketing.iter())
        } else {
            #[allow(clippy::iter_on_empty_collections)]
            // false positive, core::iter::empty() doesn't work because it has a different type
            // see https://github.com/rust-lang/rust-clippy/issues/11680
            self.rules.iter().chain([].iter())
        }
    }
}

fn is_full_match(regex: &regex::bytes::Regex, haystack: &[u8]) -> bool {
    let found = regex.find(haystack);
    #[allow(clippy::option_if_let_else)]
    match found {
        Some(m) => m.len() == haystack.len(),
        None => false,
    }
}
