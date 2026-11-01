// SPDX-FileCopyrightText: 2026 jendrikw
//
// SPDX-License-Identifier: LGPL-3.0-only

use clearurls::{Error, UrlCleaner};
use std::str::FromStr;
use url::{ParseError, Url};

#[test]
fn test_single_url() {
    let cleaner = UrlCleaner::from_embedded_rules().unwrap();

    let test = |original: &str, expected: &str| {
        let result = cleaner.clear_single_url_str(original).unwrap();
        assert_eq!(result, expected);
        let url = Url::from_str(original).unwrap();
        let result = cleaner.clear_single_url(&url).unwrap();
        assert_eq!(result.as_str(), expected);
    };

    test(
        "https://deezer.com/track/891177062?utm_source=deezer",
        "https://deezer.com/track/891177062",
    );

    test(
        "https://deezer.com/track/891177062#utm_source=deezer",
        "https://deezer.com/track/891177062",
    );

    // url encoded parameter
    test(
        "https://www.google.com/url?q=https%3A%2F%2Fpypi.org%2Fproject%2FUnalix",
        "https://pypi.org/project/Unalix",
    );

    // url encoded parameter that's not a url
    assert!(matches!(
        cleaner
            .clear_single_url_str("https://www.google.com/url?q=http%3A%2F%2F%5B%3A%3A%3A1%5D")
            .unwrap_err(),
        Error::UrlSyntax(ParseError::InvalidIpv6Address)
    ));
    assert!(matches!(
        cleaner
            .clear_single_url(
                &Url::from_str("https://www.google.com/url?q=http%3A%2F%2F%5B%3A%3A%3A1%5D")
                    .unwrap()
            )
            .unwrap_err(),
        Error::UrlSyntax(ParseError::InvalidIpv6Address)
    ));

    // double url encoded parameter

    // test cases from unalix: https://github.com/AmanoTeam/Unalix/blob/3dd238df093185390cd5ce9adc5d77aa1ee93d31/tests/test_url_clear.py
    test(
        "https://www.google.com/url?q=https%253A%252F%252Fpypi.org%252Fproject%252FUnalix",
        "https://pypi.org/project/Unalix",
    );

    test(
        "https://www.google.com/amp/s/de.statista.com/infografik/amp/22496/anzahl-der-gesamten-positiven-corona-tests-und-positivenrate/",
        "http://de.statista.com/infografik/amp/22496/anzahl-der-gesamten-positiven-corona-tests-und-positivenrate/",
    );

    test(
        "https://www.google.com/url?u=12345&adurl=www.rightstufanime.com%2Fsearch%3Fkeywords%3DSo%20I%27m%20a%20Spider%20So%20What%3F",
        "http://www.rightstufanime.com/search?keywords=So+I%27m+a+Spider+So+What%3F"
    );

    test(
        "https://www.amazon.com/gp/B08CH7RHDP/ref=as_li_ss_tl",
        "https://www.amazon.com/gp/B08CH7RHDP",
    );

    test("http://0.0.0.0/?utm_source=local", "http://0.0.0.0/");

    test(
        "https://myaccount.google.com/?utm_source=google",
        "https://myaccount.google.com/?utm_source=google",
    );

    test(
        "http://clickserve.dartsearch.net/link/click?ds_dest_url=http://g.co/",
        "https://g.co/",
    );

    test("http://example.com/?p1=&p2=", "http://example.com/?p1=&p2=");

    test(
        "http://example.com/?p1=value&p1=othervalue",
        "http://example.com/?p1=value&p1=othervalue",
    );

    test("http://example.com/?&&&&", "http://example.com/");

    test(
        "https://docs.julialang.org/en/v1/stdlib/REPL/#Key-bindings",
        "https://docs.julialang.org/en/v1/stdlib/REPL/#Key-bindings",
    );

    // test cases from clean-links: https://github.com/stringertheory/clean-links/blob/f66d6c18defc1481e3291b8c597c7d6503603764/tests/test_clean.py#L12
    test(
        "https://www.amazon.com/Kobo-Glare-Free-Touchscreen-ComfortLight-Adjustable/dp/B0BCXLQNCC/ref=pd_ci_mcx_mh_mcx_views_0?pd_rd_w=Dx5dF&content-id=amzn1.sym.225b4624-972d-4629-9040-f1bf9923dd95%3Aamzn1.symc.40e6a10e-cbc4-4fa5-81e3-4435ff64d03b&pf_rd_p=225b4624-972d-4629-9040-f1bf9923dd95&pf_rd_r=A7JSDJGYR33BN5GRCV7V&pd_rd_wg=xW6Yf&pd_rd_r=4b8a3532-9e28-4857-a929-5e572d2c765f&pd_rd_i=B0BCXLQNCC",
        "https://www.amazon.com/Kobo-Glare-Free-Touchscreen-ComfortLight-Adjustable/dp/B0BCXLQNCC",
    );

    test(
        "https://papers.ssrn.com/sol3/papers.cfm?abstract_id=1144182",
        "https://papers.ssrn.com/sol3/papers.cfm?abstract_id=1144182",
    );

    // should not be changed
    test("javascript:void(0)", "javascript:void(0)");
    test("data:,Hello%2C%20World%21", "data:,Hello%2C%20World%21");
    test(
        "data:text/plain;base64,SGVsbG8sIFdvcmxkIQ==",
        "data:text/plain;base64,SGVsbG8sIFdvcmxkIQ==",
    );
    test(
        "https://example.com/query?peers=5%B02%27",
        "https://example.com/query?peers=5%B02%27",
    );
    test(
        "https://example.com/path/123?user+name=random+user&id=123",
        "https://example.com/path/123?user+name=random+user&id=123",
    );

    // test case from lemmy: https://github.com/LemmyNet/lemmy/blob/dd033a3d243e33d3fc51070c8dadf1477748e13f/crates/utils/src/utils/markdown/mod.rs#L265
    test(
        "https://example.com/path/123?utm_content=buffercf3b2&utm_medium=social&user+name=random+user&id=123",
        "https://example.com/path/123?user+name=random+user&id=123"
    );

    // test case from url-purify: https://codeberg.org/mkljczk/url-purify/src/commit/438181ea95495c6fd6d93a7dc92c4367ec0a5904/README.md
    test(
        "https://twitter.com/Br_Nowak/status/1586618354307039234?ref_src=twsrc%5Etfw%7Ctwcamp%5Etweetembed%7Ctwterm%5E1586669502292434944%7Ctwgr%5E2432a774390f7dcbd3b885ac9570cacdb3c48c9d%7Ctwcon%5Es3_&ref_url=https%3A%2F%2Fwww.wprost.pl%2Fpolityka%2F10927933%2Fkuriozalny-wpis-malopolskiej-kurator-oswiaty-przed-1-listopada-internauci-bezlitosni.html",
        "https://twitter.com/Br_Nowak/status/1586618354307039234"
    );
}
