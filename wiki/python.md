# Python guide

[README](../README.md) | Python | [Rust](rust.md) | [Go](go.md)

The same client is reachable from Python through the
[`fmp-py-sdk`](https://pypi.org/project/fmp-py-sdk/) distribution, imported as
`fmp`. It is a Rust-backed extension for CPython 3.10 or newer (one abi3 wheel
per platform) that exposes all 271 `libfmp` methods grouped into 30 domain
namespaces. Install it with:

```console
python -m pip install fmp-py-sdk
```

## Fetch a quote

```python
from fmp import FmpClient

client = FmpClient()
rows = client.quote.short("AAPL")
print(rows[0].symbol, rows[0].price)
```

`FmpClient()` reads the `FMP_API_KEY` environment variable when `token` is
omitted; pass `token=...` to override it, or `auth_mode="none"` to ignore it.
With neither a token nor the variable, the default host raises
`FmpConfigError` naming `FMP_API_KEY`.

## Namespaces

Every `libfmp` endpoint is reachable as `client.<domain>.<method>(...)`:
required arguments are positional, optional ones keyword-only, names are
snake_case, and rows come back as typed models (`list[QuoteShort]` above).
Nested domains group their sub-namespaces, and the few open-ended endpoints
return `list[dict]` rows instead of models:

```python
income = client.statements.income.statement("AAPL", period="annual", limit=5)
valuation = client.dcf.custom("AAPL", beta=1.2, tax_rate=0.21)
rows = client.sec_filings.search_industry_classifications(symbol="AAPL")
```

The client is synchronous: each call releases the Python GIL while the async
Rust transport waits, so other threads keep running. A Python async facade is
not part of the current release.

## Responses

Rows keep the provider's wire meaning. The SDK does not guess units, scale
values, or invent a timezone.

| Wire value | Python type | What to do with it |
|------------|-------------|--------------------|
| numeric string, for example `CompanyProfile.full_time_employees` | `str`, the exact wire text | wrap with `Decimal(...)` for arithmetic |
| `Quote.timestamp` | `int`, Unix **seconds** | `datetime.fromtimestamp(ts, tz=timezone.utc)` |
| `AftermarketTrade.timestamp`, `AftermarketQuote.timestamp` | `int`, Unix **milliseconds** | divide by 1000 first |
| `YYYY-MM-DD HH:MM:SS` | naive `datetime.datetime` | the provider does not document the timezone; do not assume UTC |
| `YYYY-MM-DD` | `datetime.date` | |
| RFC 3339 timestamp, for example `TipRanksRatingSearchResult.date` | `str`, the exact wire text | `datetime.fromisoformat(...)` on Python 3.11 or newer |

```python
from datetime import datetime, timezone
from decimal import Decimal

profile = client.company.profile("AAPL")[0]
employees = Decimal(profile.full_time_employees)

quote = client.quote.full("AAPL")[0]
at = datetime.fromtimestamp(quote.timestamp, tz=timezone.utc)
```

Models are keyword-only value objects: `==` compares every field, which makes
them unhashable (no `set` members or `dict` keys; key by a field such as
`row.symbol`). `repr()` lists the fields, `__match_args__` supports positional
`case` patterns, `to_dict()` returns a plain `dict` with nested models as dicts,
and pickling rebuilds the row by keyword. JSON-number fields are `int | float`.

```python
rows = client.quote.short("AAPL")
by_symbol = {row.symbol: row for row in rows}
payload = [row.to_dict() for row in rows]
```

## Errors

Local argument validation raises `FmpValidationError` before any request;
provider and transport failures map to the hierarchy under `fmp.errors`, with
`FmpError` as the base class. Bodies attached to errors are redacted before
they reach Python.

## Closing a client

`close()` releases the client's pooled HTTP connections. Use the client as a
context manager to close it deterministically:

```python
from fmp import FmpClient

with FmpClient() as client:
    rows = client.quote.short("AAPL")
```

After `close()`, every call raises `FmpConfigError("the client is closed")`,
including calls through a namespace such as `client.quote` fetched before the
close; calls already in flight finish normally. Closing twice is a no-op, and
`__exit__` never swallows an exception from the block. The process-wide
runtime is shared by every client and stays up.

A client that is never closed releases its pooled connections lazily, some
time after the client and its namespaces are garbage-collected.

## Custom routers and proxies

Endpoint code is independent of transport configuration. The client supports
a custom base URL and path prefix, no authentication, FMP header or query
authentication, bearer authentication, custom secret headers or query
parameters, and arbitrary default headers.

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
rows = client.quote.short("AAPL")
```

`auth_mode="none"` supports credential-free local or trusted routers.
Redirect following is either disabled or restricted to the same origin so
credentials are not forwarded across origins.

## Reference

The [fmp-py README](../crates/fmp-py/README.md) is the text shipped to PyPI.
It covers:

- the full namespace table and the nested `statements` sub-namespaces;
- typed, dynamic, and binary responses, and the `fmp.errors` hierarchy;
- working with models: keyword-only constructors, value equality, `to_dict()`;
- typing: the `py.typed` marker, the `.pyi` stubs generated by `pyo3-stub-gen`,
  `Literal` types for closed string arguments, and the stubtest gate;
- secret URLs on financial-report rows and how they are redacted;
- regenerating the models, namespaces, stubs, and public packages from the registry.
