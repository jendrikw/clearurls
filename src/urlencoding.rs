// SPDX-FileCopyrightText: 2026 jendrikw
//
// SPDX-License-Identifier: LGPL-3.0-only

use alloc::borrow::Cow;
use alloc::vec::Vec;
use alloc::string::String;
use percent_encoding::{
    percent_decode, percent_decode_str, percent_encode, AsciiSet, NON_ALPHANUMERIC,
};

pub(crate) type SearchParam<'a> = (Cow<'a, [u8]>, Option<Cow<'a, [u8]>>);

const FORM_URL_ENCODED_ASCII_SET: AsciiSet = NON_ALPHANUMERIC
    .remove(b'*')
    .remove(b'-')
    .remove(b'.')
    .remove(b'_');

#[allow(clippy::option_if_let_else)]
pub fn parse_query_parameters(s: &[u8]) -> Vec<SearchParam<'_>> {
    let mut result = Vec::new();
    for kv in s.split(|b| *b == b'&') {
        if kv.is_empty() {
            break;
        }
        let mut split2 = kv.splitn(2, |b| *b == b'=');
        let name = split2.next().unwrap();
        let name = parse_param(name);
        let value = split2.next().map(parse_param);
        result.push((name, value));
    }
    result
}

fn parse_param(input: &[u8]) -> Cow<'_, [u8]> {
    if !input.contains(&b'+') {
        return Cow::from(percent_decode(input));
    }

    let mut output = Vec::with_capacity(input.len());
    let mut first = true;
    for part in input.split(|b| *b == b'+') {
        if !first {
            output.push(b' ');
            first = false;
        }
        output.extend(percent_decode(part));
    }
    Cow::Owned(output)
}

pub fn repeatedly_urldecode(s: &str) -> Cow<'_, str> {
    let mut before = Cow::Borrowed(s);
    loop {
        let after = Cow::from(percent_decode_str(&before));
        match after {
            Cow::Borrowed(after) => {
                // unchanged, so return now
                return match core::str::from_utf8(after) {
                    Ok(after) => {
                        if after.starts_with("http") {
                            before
                        } else {
                            Cow::Owned(["http://", after].join(""))
                        }
                    }
                    Err(_) => before,
                };
            }
            Cow::Owned(after) => match String::from_utf8(after) {
                Ok(after) => {
                    before = Cow::Owned(after);
                }
                Err(_) => {
                    return before;
                }
            },
        }
    }
}

pub fn search_params_to_string(params: &[SearchParam<'_>]) -> Option<String> {
    if params.is_empty() {
        return None;
    }
    let mut res = String::new();
    for (i, (k, v)) in params.iter().enumerate() {
        if i > 0 {
            res.push('&');
        }
        res.extend(percent_encode(k.as_ref(), &FORM_URL_ENCODED_ASCII_SET));
        if let Some(v) = v {
            res.push('=');
            res.extend(percent_encode(v.as_ref(), &FORM_URL_ENCODED_ASCII_SET));
        }
    }
    Some(res)
}
