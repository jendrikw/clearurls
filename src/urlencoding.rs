// SPDX-FileCopyrightText: 2026 jendrikw
//
// SPDX-License-Identifier: LGPL-3.0-only

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use percent_encoding::{percent_decode, percent_decode_str};
use url::form_urlencoded::byte_serialize;

pub(crate) type SearchParam<'a> = (Cow<'a, [u8]>, Option<Cow<'a, [u8]>>);

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
    for (i, part) in input.split(|b| *b == b'+').enumerate() {
        if i > 0 {
            output.push(b' ');
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
                debug_assert_eq!(before.as_bytes(), after);
                return if before.starts_with("http") {
                    before
                } else {
                    Cow::Owned(["http://", &before].join(""))
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
        res.extend(byte_serialize(k));
        if let Some(v) = v {
            res.push('=');
            res.extend(byte_serialize(v));
        }
    }
    Some(res)
}

#[cfg(test)]
mod tests {
    use super::repeatedly_urldecode;

    #[test]
    fn test_invalid_utf8_after_percent_decode() {
        let input = "%FF";
        let result = repeatedly_urldecode(input);

        assert_eq!(result, "%FF");
    }
}
