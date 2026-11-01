// SPDX-FileCopyrightText: 2026 jendrikw
//
// SPDX-License-Identifier: LGPL-3.0-only

pub(crate) trait GenericRegex: Sized {
    type Builder: GenericRegexBuilder<Self>;
}

impl GenericRegex for regex::Regex {
    type Builder = regex::RegexBuilder;
}

impl GenericRegex for regex::bytes::Regex {
    type Builder = regex::bytes::RegexBuilder;
}

pub(crate) trait GenericRegexBuilder<R> {
    fn build(pattern: &str) -> Result<R, regex::Error>;
}

impl GenericRegexBuilder<regex::bytes::Regex> for regex::bytes::RegexBuilder {
    fn build(pattern: &str) -> Result<regex::bytes::Regex, regex::Error> {
        Self::new(pattern).case_insensitive(true).build()
    }
}

impl GenericRegexBuilder<regex::Regex> for regex::RegexBuilder {
    fn build(pattern: &str) -> Result<regex::Regex, regex::Error> {
        Self::new(pattern).case_insensitive(true).build()
    }
}
