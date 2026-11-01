// SPDX-FileCopyrightText: 2026 jendrikw
//
// SPDX-License-Identifier: LGPL-3.0-only

#[cfg(feature = "linkify")]
#[test]
fn test_linkify() {
    use clearurls::UrlCleaner;

    let cleaner = UrlCleaner::from_embedded_rules().unwrap();

    let test = |msg: &str, input: &str, expected: &str| {
        let result = cleaner
            .clear_text(input)
            .unwrap_or_else(|e| panic!("error in test {msg}: {e:?}"));

        assert_eq!(
            result, expected,
            "Testing {msg}, with original input '{input}'"
        );
    };

    test(
        "0 links",
        "This is a markdown text.",
        "This is a markdown text.",
    );

    test(
        "2 links",
        "This is a [markdown link](http://example.com/?&&&&), and another: http://example.com?utm_source=1",
        "This is a [markdown link](http://example.com/), and another: http://example.com/",
    );

    test(
        "redirection",
        "test: https://google.co.uk/url?foo=bar&q=http://example.com/%F0",
        "test: http://example.com/%F0",
    );
}
