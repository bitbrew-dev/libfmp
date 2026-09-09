# libfmp

`libfmp` is a Rust-first client for the [Financial Modeling Prep (FMP)](https://financialmodelingprep.com/) data API, with synchronous Python bindings distributed as `fmp-py-sdk` and imported as `fmp`.

The Rust client implements all 276 endpoint entries in the repository's
captured API documentation oracle. This is coverage of that pinned oracle, not
a claim that every endpoint currently or historically offered by the provider
is covered. The Python facade currently exposes 1 of those 276 entries.

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

The Rust builder never reads the environment on its own. When the key lives in
`FMP_API_KEY`, `Authentication::fmp_header_from_env()` returns the same header
authentication as `Option<Authentication>` (`None` when the variable is unset,
empty, or whitespace-only), so the `env::var` line above is the explicit form.

The Rust client is async. Endpoint descriptors, typed queries, response models,
and `Client` methods cover all 276 documented oracle entries. `quote_short`,
for example, preserves FMP's bare-array response as `Vec<QuoteShort>`, including
empty and multi-row responses.

## Python

Install the distribution and import the public package:

```console
python -m pip install fmp-py-sdk
```

```python
from fmp import FmpClient

client = FmpClient()
rows = client.quote_short("AAPL")
print(rows)
```

`FmpClient()` reads the `FMP_API_KEY` environment variable when `token` is
omitted; pass `token=...` to override it, or `auth_mode="none"` to ignore it.
With neither a token nor the variable, the default host raises
`FmpConfigError` naming `FMP_API_KEY`.

The Python facade is synchronous and currently implements only the documented
`quote_short` endpoint (1/276). It returns `list[QuoteShort]`; while waiting for
the async Rust transport it releases the Python GIL. The other Python facades
remain planned work and should not be inferred from Rust coverage.

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

The repository ships inline Rust documentation plus domain-aligned native
Python modules, documentation/source `.py` shims, hand-maintained `.pyi` stubs,
and a `py.typed` marker.

## Release status

This project is available under the [MIT License](LICENSE). See [docs/releasing.md](docs/releasing.md) for the remaining release checks.
