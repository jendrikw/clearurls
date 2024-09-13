#[cfg(feature = "linkify")]
#[test]
fn test_linkify() {
    use clearurls::Error;
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

    let err = cleaner.clear_text("This is a [markdown link](http://example.com/?&&&&), and another: https://google.co.uk/url?foo=bar&q=http%F0");
    assert!(matches!(
        err.unwrap_err()[..],
        [Error::PercentDecodeUtf8Error(_)]
    ));
}
