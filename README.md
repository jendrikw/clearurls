# clearurls

![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/jendrikw/clearurls/rust.yml)
[![Crate](https://img.shields.io/crates/v/clearurls.svg)](https://crates.io/crates/clearurls)
[![Docs](https://docs.rs/clearurls/badge.svg)](https://docs.rs/clearurls)

Bringing the power of the [ClearURLs](https://clearurls.xyz/) rules to Rust.
Easily remove tracking parameters and other nuisance from URLs with a simple API:

```rust
use clearurls::UrlCleaner;
fn main() -> Result<(), clearurls::Error> {
    let cleaner = UrlCleaner::from_embedded_rules()?;
    let res = cleaner.clear_single_url_str("https://example.com/test?utm_source=abc")?;
    assert_eq!(res, "https://example.com/test");
    Ok(())
}
```

## Crate Features

There is a `std` feature (enabled by default) to include utility functions to read from files,
but the core logic doesn't depend on that and the crate is perfectly usable without `std`.

## Acknowledgements

`data.minify.json` was downloaded from <https://github.com/ClearURLs/Rules>

## License

data.minify.json file is from <https://github.com/ClearURLs/Rules>
Testcases are from

- <https://github.com/icealtria/Unalix-Rev/blob/master/src/tests>
- <https://github.com/stringertheory/clean-links/tree/main/tests>
