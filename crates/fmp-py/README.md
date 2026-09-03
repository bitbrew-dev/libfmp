# fmp-py-sdk

`fmp-py-sdk` is the Python distribution for the Rust-backed `fmp` package, a typed client for the [Financial Modeling Prep (FMP)](https://financialmodelingprep.com/) data API.

The Python facade currently exposes 1 of the 276 endpoint entries in the
repository's captured API documentation oracle: the documented
`GET /stable/quote-short?symbol=...` endpoint. The remaining Python facades are
planned work; Rust's broader endpoint coverage does not imply Python parity.

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

`quote_short` returns `list[QuoteShort]`, preserving empty and multi-row provider responses. The public native modules follow a conventional HTTP SDK surface: `FmpClient` lives in `fmp.client`, the exception hierarchy lives in `fmp.errors`, and response models live in endpoint domains such as `fmp.quote`. Every public type also remains available from the package root. `FmpClient` is synchronous; it releases the Python GIL while its async Rust transport waits.

## Custom router or proxy

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

Available auth modes are `none`, `fmp_header`, `fmp_query`, `bearer`, `custom_header`, and `custom_query`. `auth_mode="none"` supports credential-free local or trusted routers. Redirect following is either disabled or same-origin only.

The package supports CPython 3.9 or newer through Python's stable ABI. Native
public modules are paired with documentation/source `.py` shims and
hand-maintained `.pyi` contracts, plus a `py.typed` marker for type checkers.
Transport, configuration, and runtime internals are intentionally not exposed
as Python modules.

This project is available under the [MIT License](LICENSE).
