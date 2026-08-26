# fmp-py-sdk

`fmp-py-sdk` is the Python distribution for the Rust-backed `fmp_py` package, a typed client for the [Financial Modeling Prep (FMP)](https://financialmodelingprep.com/) data API.

Version 0.1 is intentionally a name-release walking skeleton. It supports only the documented `GET /stable/quote-short?symbol=...` endpoint and does not claim broad endpoint coverage.

```console
python -m pip install fmp-py-sdk
```

```python
import os

from fmp_py import FmpClient

client = FmpClient(token=os.environ["FMP_API_KEY"])
rows = client.quote_short("AAPL")
print(rows)
```

`quote_short` returns `list[QuoteShort]`, preserving empty and multi-row provider responses. `FmpClient` is synchronous; it releases the Python GIL while its async Rust transport waits.

## Custom router or proxy

```python
import os

from fmp_py import FmpClient

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

The package supports CPython 3.9 or newer through Python's stable ABI. It includes `.pyi` stubs and a `py.typed` marker for type checkers.

The project license is not yet selected, so publication is blocked until consistent license metadata is added.
