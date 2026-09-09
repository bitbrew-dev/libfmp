# libfmp

`libfmp` is an async, typed Rust client for the [Financial Modeling Prep (FMP)](https://financialmodelingprep.com/) data API.

The crate implements all 276 endpoint entries in the repository's captured API
documentation oracle through transport-independent descriptors, typed queries,
response models, and `Client` methods. This is coverage of that pinned oracle,
not a claim that every endpoint currently or historically offered by the
provider is covered. The separately distributed synchronous Python facade
currently exposes 1 of those 276 entries.

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

The documented bare JSON array remains a `Vec<QuoteShort>`; empty and multi-row responses keep their original shape.

## Custom router or proxy

Transport choices do not change endpoint code. A caller can select a custom base URL and path prefix, no auth, FMP header or query auth, bearer auth, custom secret header or query auth, and arbitrary default headers.

```rust
use std::env;

use libfmp::{Client, config::Authentication, types::Ticker};

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = Client::builder()
    .base_url(env::var("FMP_PROXY_BASE_URL")?)
    .path_prefix("router/stable")
    .authentication(Authentication::custom_header(
        "X-Proxy-Token",
        Some("Bearer ".to_owned()),
        env::var("FMP_PROXY_TOKEN")?,
    ))
    .default_header("X-Tenant", env::var("FMP_TENANT")?)
    .build()?;
let rows = client.quote_short(Ticker::new("AAPL")?).await?;
# let _ = rows;
# Ok(())
# }
```

Rust 1.96 or newer is supported. The repository pins Rust 1.96.0 for development and release validation. This project is available under the [MIT License](LICENSE).
