# Rust guide

[README](../README.md) | [Python](python.md) | Rust | [Go](go.md)

`libfmp` is the async, typed Rust core. Endpoint descriptors, typed queries,
response models, and `Client` methods cover every documented oracle entry:
271 methods across 30 domains. Start with
[installation](../README.md#install), then the
[API reference on docs.rs](https://docs.rs/libfmp).

## Fetch a quote

```rust
use std::env;

use libfmp::{Client, config::Authentication, types::Ticker};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder()
        .authentication(Authentication::fmp_header(env::var("FMP_API_KEY")?))
        .build()?;
    let rows = client.quote_short(Ticker::new("AAPL")?).await?;
    println!("{rows:#?}");
    Ok(())
}
```

The Rust builder never reads the environment on its own. When the key lives in
`FMP_API_KEY`, `Authentication::fmp_header_from_env()` returns the same header
authentication as `Option<Authentication>` (`None` when the variable is unset,
empty, or whitespace-only), so the `env::var` line above is the explicit form.

The client is async: every endpoint method returns a future, so it runs under
an async runtime such as `tokio`. `quote_short`, for example, preserves FMP's
bare-array response as `Vec<QuoteShort>`, including empty and multi-row
responses.

## Custom routers and proxies

Endpoint code is independent of transport configuration. The builder accepts
a custom base URL and path prefix, no authentication, FMP header or query
authentication, bearer authentication, custom secret headers or query
parameters, and arbitrary default headers. Redirect following is either
disabled or restricted to the same origin, so credentials are never forwarded
across origins. The crate README shows the full proxy builder in
[Custom router or proxy](../crates/libfmp/README.md#custom-router-or-proxy).

## Reference

- [API reference](https://docs.rs/libfmp) on docs.rs.
- [Crate README](../crates/libfmp/README.md): the text packaged with the crate.
- [Core source](../crates/libfmp/src/): client, configuration, transport, and one module per endpoint domain.
- [Tests](../crates/libfmp/tests/) and the [recorded wire fixtures](../crates/libfmp/tests/fixtures/) they decode.
