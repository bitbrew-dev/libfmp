# libfmp

`libfmp` is a Rust-first client for the [Financial Modeling Prep (FMP)](https://financialmodelingprep.com/) data API, with synchronous Python bindings distributed as `fmp-py-sdk` and imported as `fmp`.

Version 0.1 is intentionally a name-release walking skeleton. It supports only the documented `GET /stable/quote-short?symbol=...` endpoint. It does not yet promise broad FMP endpoint coverage.

## Rust

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

The Rust client is async. `quote_short` preserves FMP's bare-array response as `Vec<QuoteShort>`, including empty and multi-row responses.

## Python

Install the distribution and import the underscore-named package:

```console
python -m pip install fmp-py-sdk
```

```python
import os

from fmp import FmpClient

client = FmpClient(token=os.environ["FMP_API_KEY"])
rows = client.quote_short("AAPL")
print(rows)
```

The Python facade is synchronous and returns `list[QuoteShort]`. While waiting for the async Rust transport it releases the Python GIL.

## Custom routers and proxies

Endpoint code is independent of transport configuration. Both clients support a custom base URL and path prefix, no authentication, FMP header or query authentication, bearer authentication, custom secret headers or query parameters, and arbitrary default headers.

```python
import os

from fmp import FmpClient

client = FmpClient(
    base_url=os.environ["FMP_PROXY_BASE_URL"],
    path_prefix="router/stable",
    auth_mode="custom_header",
    auth_name="X-Proxy-Token",
    auth_prefix="Bearer ",
    token=os.environ["FMP_PROXY_TOKEN"],
    headers={"X-Tenant": os.environ["FMP_TENANT"]},
)
rows = client.quote_short("AAPL")
```

`auth_mode="none"` supports credential-free local or trusted routers. Redirect following is either disabled or restricted to the same origin so credentials are not forwarded across origins.

## Support policy

- Rust: 1.96 or newer; the repository pins Rust 1.96.0 for development and release validation.
- Python: CPython 3.9 or newer, using the stable ABI from Python 3.9.
- Package version: the Cargo workspace version is the single source used by both the `libfmp` crate and the `fmp-py-sdk` wheel.

The repository ships inline Rust documentation plus Python `.pyi` stubs and a `py.typed` marker.

## Release status

This project is available under the [MIT License](LICENSE). See [docs/releasing.md](docs/releasing.md) for the remaining release checks.
