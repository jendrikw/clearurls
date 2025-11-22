#[cfg(feature = "markdown-it")]
#[test]
fn test_markdown() {
    use clearurls::Error;
    use clearurls::UrlCleaner;
    use markdown_it::MarkdownIt;

    static SINGLE_BLACK_PIXEL: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAAXNSR0IArs4c6QAAAAlwSFlzAAAWJQAAFiUBSVIk8AAAABNJREFUCB1jZGBg+A/EDEwgAgQADigBA//q6GsAAAAASUVORK5CYII%3D";

    let mut parser = MarkdownIt::new();
    markdown_it::plugins::cmark::add(&mut parser);
    markdown_it::plugins::extra::linkify::add(&mut parser);
    let cleaner = UrlCleaner::from_embedded_rules().unwrap();

    let test = |msg: &str, input: String, expected: String| {
        let mut node = parser.parse(&input);
        cleaner
            .clear_markdown(&mut node)
            .unwrap_or_else(|e| panic!("error in test {msg}: {e:?}"));
        let result = node.xrender();

        assert_eq!(
            result, expected,
            "Testing {msg}, with original input '{input}', node: {node:?}"
        );
    };

    test(
        "angle bracket",
        "<ftp://example.com/test/?utm_source=abc>".to_string(),
        "<p><a href=\"ftp://example.com/test/\">ftp://example.com/test/</a></p>\n".to_string(),
    );
    test(
        "angle bracket",
        format!("<{SINGLE_BLACK_PIXEL}>"),
        format!("<p><a href=\"{SINGLE_BLACK_PIXEL}\">{SINGLE_BLACK_PIXEL}</a></p>\n"),
    );
    test(
        "links",
        "[Goodreads](https://goodreads.com?qid=1 \"title\")".to_string(),
        "<p><a href=\"https://goodreads.com/\" title=\"title\">Goodreads</a></p>\n".to_string(),
    );
    test(
        "links",
        format!("[data url]({SINGLE_BLACK_PIXEL})"),
        format!("<p><a href=\"{SINGLE_BLACK_PIXEL}\">data url</a></p>\n"),
    );
    test(
        "images",
        "![My linked image](https://duckduckgo.com/l/abc?uddg=http%3A%2F%2Fexample.com%2Fimage.png \"image alt text\")".to_string(),
        "<p><img src=\"http://example.com/image.png\" alt=\"My linked image\" title=\"image alt text\" /></p>\n".to_string()
    );
    test(
        "images",
        format!("![My linked image]({SINGLE_BLACK_PIXEL})"),
        format!("<p><img src=\"{SINGLE_BLACK_PIXEL}\" alt=\"My linked image\" /></p>\n"),
    );

    let err = cleaner
        .clear_markdown(&mut parser.parse("<ftp://example.%com>"))
        .unwrap_err();
    assert!(matches!(err[..], [Error::UrlSyntax(_)]));
}
