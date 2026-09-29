# fmp-py-sdk

`fmp-py-sdk` is the Python distribution for the Rust-backed `fmp` package, a typed client for the [Financial Modeling Prep (FMP)](https://financialmodelingprep.com/) data API.

The client exposes every `libfmp` endpoint method: 271 methods grouped into 30
domain namespaces on `FmpClient`. Each namespace is generated from a registry
that is validated against the real `libfmp` signatures, so the Python surface
cannot drift from the Rust one.

```console
python -m pip install fmp-py-sdk
```

```python
from fmp import FmpClient

client = FmpClient()
rows = client.quote.short("AAPL")
print(rows[0].symbol, rows[0].price)
```

`FmpClient()` reads the `FMP_API_KEY` environment variable when `token` is
omitted (unset, empty, or whitespace-only counts as absent); an explicit
`token=...` always wins, and `auth_mode="none"` ignores the variable. With
neither a token nor the variable, the default host raises `FmpConfigError`
naming `FMP_API_KEY`, while a custom `base_url` selects no auth.

`FmpClient` is synchronous: each call releases the Python GIL while the async
Rust transport waits, so threads keep running. A Python async facade is not
part of this release.

## Requirements

- CPython 3.10 or newer (`abi3-py310`: one wheel per platform covers every
  supported interpreter).
- Rust is only needed to build from source; a built wheel is self-contained.
- Do not install the unrelated PyPI distribution named `fmp` in the same
  environment: it also provides an `fmp` import package, and the two collide.

## Namespaces

Endpoints live under one attribute per domain, and nested domains such as
`statements` group their sub-namespaces. Required arguments are positional,
optional ones are keyword-only, and every name is snake_case: the FMP wire
casing (`sicCode`, `linkXlsx`) never reaches Python.

```python
from fmp import FmpClient

client = FmpClient()

quotes = client.quote.short("AAPL")
income = client.statements.income.statement("AAPL", period="annual", limit=5)
valuation = client.dcf.custom(
    "AAPL", beta=1.2, tax_rate=0.21, long_term_growth_rate=4.0
)
```

| Namespace | Namespace | Namespace |
|-----------|-----------|-----------|
| `analyst` | `esg` | `market_hours` |
| `bulk` | `forex` | `news` |
| `calendar` | `fundraising` | `quote` |
| `chart` | `funds` | `screener` |
| `commitment_of_traders` | `indexes` | `search` |
| `commodities` | `insider_trading` | `sec_filings` |
| `company` | `institutional_ownership` | `statements` |
| `congressional` | `market` | `technical_indicators` |
| `crypto` | `dcf` | `tipranks` |
| `directory` | `economics` | `transcripts` |

`statements` nests `as_reported`, `balance`, `cash_flow`, `growth`, `income`,
`metrics`, `ratios`, `reports`, `segmentation`, and `summaries`.

## Responses

- Typed rows: most methods return `list[Model]`, where each model is a
  generated, immutable, picklable class living in the domain package (for
  example `fmp.quote.QuoteShort` or `fmp.statements.income.IncomeStatement`).
  Dates and naive timestamps are `datetime.date` and `datetime.datetime`;
  RFC 3339 timestamps (for example `TipRanksRatingSearchResult.date`) stay
  `str` with the exact wire text. Shared Rust response types map to one shared
  Python class rather than a copy per endpoint.
- Models behave like frozen value objects: see [Working with models](#working-with-models).
- Numeric strings (for example `CompanyProfile.full_time_employees`) stay
  `str` with the exact wire text; wrap them with `decimal.Decimal(...)` for
  arithmetic.
- Unix timestamps are `int` in the unit the provider sends: `Quote.timestamp`
  is seconds, while `AftermarketTrade.timestamp` and
  `AftermarketQuote.timestamp` are milliseconds.
- Naive `datetime.datetime` values carry no timezone because the provider
  does not document one; the binding does not attach or assume UTC.
- Dynamic rows: endpoints whose documented shape is open-ended return
  `list[dict[str, Any]]` with the raw provider keys, for example
  `client.sec_filings.search_industry_classifications(symbol="AAPL")`. A
  dynamic field inside a typed model (such as `FinancialReportJson.sections`)
  is exposed as `Any`. A JSON-number field (for example
  `CotReport.pct_of_open_interest_all`) is `int | float`: an `int` when the
  provider sent an integer literal (exact at any size), a `float` otherwise.
- Binary bodies: `client.statements.reports.xlsx("AAPL", 2022, "FY")` returns
  a single `fmp.BinaryPayload` instead of a list. `data` is the body as
  `bytes`, alongside `content_type`, `content_disposition`, and `byte_len`.

## Working with models

```python
from fmp.quote import QuoteShort

row = QuoteShort(symbol="AAPL", price=232.8, change=1.2, volume=41_000_000.0)
row == QuoteShort(**row.to_dict())  # True: equality compares every field
row.to_dict()                       # {"symbol": "AAPL", "price": 232.8, ...}
repr(row)                           # "QuoteShort(symbol='AAPL', price=232.8, ...)"

match row:
    case QuoteShort(symbol, price):  # positional patterns follow __match_args__
        print(symbol, price)
```

- Constructors are keyword-only. Every model can be built by hand (for tests
  or fixtures) with the attribute names as keywords.
- `==` compares by value. Because of that, models are **unhashable**
  (`__hash__` is `None`, as for any Python class that defines `__eq__`): they
  cannot be `set` members or `dict` keys. Key a dict by a field instead, for
  example `{row.symbol: row for row in rows}`.
- `repr()` lists every attribute in constructor order, and `__match_args__`
  holds the same names for positional `case` patterns.
- `to_dict()` returns a plain `dict` keyed by attribute name; nested models
  become dicts and lists of models become lists of dicts. JSON-number fields
  are `int | float`, dynamic fields are the decoded JSON.
- Pickling and `copy.copy` rebuild the row by keyword through
  `__getnewargs_ex__`.
- Secret fields are not attributes: they are left out of `repr()`,
  `__match_args__`, and `to_dict()` (see [Secret URLs](#secret-urls)).

## Validation and errors

Arguments are validated locally before any request: an empty ticker, a
malformed date, or a non-finite float raises `FmpValidationError` whose
message starts with the argument name, and nothing is sent. The exception
hierarchy lives in `fmp.errors` (every class is also exported from the
package root):

| Exception | Raised when |
|-----------|-------------|
| `FmpError` | base class; carries `category`, `endpoint`, `status`, `body`, `body_truncated`, `decode_path`, `decode_kind` |
| `FmpConfigError` | the client cannot be built (missing key, bad URL, insecure auth) |
| `FmpValidationError` | an argument is rejected before the request |
| `FmpTransportError` | the request never produced a response |
| `FmpStatusError` | the provider answered with a non-success status |
| `FmpDecodeError` | the body could not be decoded into the documented shape; `decode_path` names the member (for example `[37].beta`) and `decode_kind` the reason, never the value |

```python
from fmp import FmpClient
from fmp.errors import FmpStatusError

client = FmpClient()
try:
    client.quote.short("AAPL")
except FmpStatusError as error:
    print(error.status, error.endpoint, error.body)
```

Bodies attached to errors are redacted before they reach Python: an echoed
`apikey` query value shows as `[REDACTED]`.

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
rows = client.quote.short("AAPL")
```

Available auth modes are `none`, `fmp_header`, `fmp_query`, `bearer`, `custom_header`, and `custom_query`. `auth_mode="none"` supports credential-free local or trusted routers. Redirect following is either disabled or same-origin only.

## Typing

The package ships a `py.typed` marker, and every native module has a `.pyi`
stub generated by `pyo3-stub-gen` from the Rust signatures, so `pyright` and
`mypy` see the exact method names, keyword-only arguments, and return types.
Closed string vocabularies are `typing.Literal` types: `period` arguments
(`"Q1"`..`"FY"`, `"annual"`, `"quarter"`), technical-indicator `timeframe`,
revenue-segmentation `structure`, `FmpClient(auth_mode=...)`, and model fields
such as `IncomeStatement.period` or the `"Y"`/`"N"` flags. The stubs list the
canonical spellings; at runtime `period`, `timeframe`, and `structure` stay
case-insensitive and accept the documented aliases (for example `"quarterly"`),
which a type checker rejects, while `auth_mode` must match exactly. Open vocabularies (form types, economic indicator names) stay `str`.
`FmpClient(headers=...)` takes any `Mapping[str, str]`.

The test suite runs `python -m mypy.stubtest fmp` and `mypy` on
`tests/typecheck_contract.py` whenever `mypy` is installed in the test
environment, so a stub that drifts from the built extension fails `pytest`.
The public modules follow a conventional HTTP SDK layout: `FmpClient` lives in
`fmp.client`, the exception hierarchy in `fmp.errors`, and the models and
namespace classes in the domain packages such as `fmp.quote` and
`fmp.statements.income`. Transport, configuration, and runtime internals are
intentionally not exposed as Python modules.

## Secret URLs

`FinancialReportDate.link_json` / `link_xlsx` (the `fmp.statements.reports` model
listing available financial reports) are download links that embed your API key. The Python model treats them the
way `FmpClient` treats `danger_allow_insecure_authentication`: the unsafe path
exists, but it is conspicuous.

- The links are not attributes. Read one with `row.expose_secret_url_json()` or
  `row.expose_secret_url_xlsx()` and treat the value as a credential.
- `repr(row)` and `str(row)` print `link_json=[REDACTED URL]`, so a stray
  `print` or log line never leaks the key.
- For local debugging only, `fmp.set_reveal_secret_urls(True)` reveals the URLs
  in `repr()` process-wide (`fmp.reveal_secret_urls()` reads the flag back).
  Setting `FMP_REVEAL_SECRET_URLS=1` (also `true` / `yes`, case-insensitive)
  before the first use turns it on at start-up.
- Pickles of the model contain the real URLs: `pickle.dumps` must round-trip
  the value, so store them as carefully as the key itself.
- `to_dict()` and `__match_args__` leave the links out; `==` still compares
  them.

## Regenerating the response models

The `pyo3` response models under `crates/fmp-py/src/models/` are generated
from the `libfmp` response structs by the `gen_models` binary in the sibling
`crates/fmp-py-gen` crate. Run it from the repository root:

```console
cargo run -p fmp-py-gen --bin gen_models
```

The generator only depends on `syn`, not on `fmp-py`, so it still builds and
runs when `fmp-py` fails to compile against a changed `libfmp`. Regeneration
must be idempotent: `git status crates/fmp-py/src/models` is clean after a
second run.

## Validating the endpoint registry

The Python namespaces are described by one TOML file per domain under
`crates/fmp-py-gen/registry/` (`quote.toml` for `client.quote`). Each entry
names the Python method, the `libfmp` `Client` method, its query type, the
constructor arguments and optional `with_*` setters with their arg kinds, and
the generated response model. Validate the registry against the real `libfmp`
signatures and the generated models from the repository root:

```console
cargo run -p fmp-py-gen --bin registry_check
```

The check parses `crates/libfmp/src/endpoints/**` with `syn`, so an unknown
method, a mismatched query type, an unknown arg kind, a setter that does not
exist, or a missing model file fails with the file and entry named. Query
types emitted by `macro_rules!` are recovered by expanding the macro; the
report says whether each entry was verified directly, through a macro, or
trusted because its constructor could not be seen, and ends with the total
(`registry ok: 271 verified, 0 trusted`). Pass a directory argument to check
a different registry tree.

## Regenerating the endpoint namespaces

The namespace classes under `crates/fmp-py/src/namespaces/` (`QuoteNamespace`
for `client.quote`) are generated from the same registry by the
`gen_namespaces` binary, which validates the registry first and emits nothing
on any error. Run it from the repository root:

```console
cargo run -p fmp-py-gen --bin gen_namespaces
```

Flat domains become `namespaces/<domain>.rs`; a nested path such as
`statements.income` becomes `namespaces/statements/mod.rs` (the parent with a
getter per sub-namespace) plus `namespaces/statements/income.rs`. Required
arguments are positional, optional ones keyword-only, and an argument named
after a Python keyword (`from`) is spelled with a trailing underscore
(`from_`) on the Python side while the `libfmp` setter keeps its name.
Entries marked `binary = true` (the endpoints whose `libfmp` method returns
`BinaryResponse`, such as the XLSX financial report download) return a single
`BinaryPayload` instead of a list of models; entries marked
`response = "dynamic"` return `list[dict[str, Any]]`. `BinaryPayload` is
hand-written in `src/binary.rs` and exported from the package root as
`fmp.BinaryPayload`.

The same run emits the wiring that binds the generated code into the
extension, so adding a domain never edits `lib.rs` or `client.rs`:

| File | Contents |
|------|----------|
| `src/namespaces/mod.rs` | one `mod` declaration per domain |
| `src/registration.rs` | `register_namespaces`: every `fmp._native.<path>` submodule with its models (read back from `src/models/**` with `syn`) and namespace classes, published in `sys.modules` |
| `src/client_namespaces.rs` | one `FmpClient` getter per domain, a second `#[pymethods]` block (pyo3's `multiple-pymethods` feature) |
| `src/facade_domains.rs` | a wildcard `reexport_module_members!` per `fmp._native.<path>`, which is what writes the public `python/fmp/<path>/__init__.py` packages |

The hand-written `src/facade.rs` keeps only the top-level `fmp` surface.

## Regenerating the Go SDK

The Go module at `sdk/go` (ADR 0030) is generated from the same registry, the
wire contract read from `crates/libfmp/src/endpoints/**`, and the response
structs, by the `gen_go` binary. The three generators, from the repository
root:

```console
cargo run -p fmp-py-gen --bin gen_models      # fmp-py response models
cargo run -p fmp-py-gen --bin gen_namespaces  # fmp-py endpoint namespaces
cargo run -p fmp-py-gen --bin gen_go          # sdk/go, every generated domain
```

`gen_go --domain <name>` (repeatable) generates the named domains; with no
argument it regenerates every domain that already carries the generated
header; `--all` regenerates all 30. It writes `sdk/go/<domain>_models.go`,
`sdk/go/<domain>.go`, and the shared `queries.go` and `namespaces.go`, all
through `gofmt`, so a second run leaves `git status` clean; the
`scripts/check_go_sdk.sh` gate proves it. A Rust type or argument kind the
generator does not know fails the run naming the struct and field (or the
`query.go` helper to add); nothing is emitted in that case.

## Regenerating the stubs and public packages

The `.pyi` stubs under `python/fmp/_native/` and the public
`python/fmp/<path>/__init__.py` packages are written by `pyo3-stub-gen`
through the `stub_gen` binary, from the repository root:

```console
cargo run -p fmp-py --bin stub_gen
```

`stub_gen` post-processes each `__init__.pyi` (absolute imports, fully
qualified module references so a method named like a module cannot shadow it,
the typeshed `__hash__: ClassVar[None]` spelling, no `# ruff: noqa` header, `ruff format --isolated --line-length 88` in `.py`
mode) so a run on an unchanged tree leaves `git status` clean. It needs
`ruff` at the version pinned in this crate's `.pre-commit-config.yaml` and as
`RUFF_VERSION` in `src/bin/stub_gen.rs` (currently 0.15.12, the two must
agree): either that exact `ruff` on `PATH` or `uvx`, which fetches it. Run it
after `gen_models` or `gen_namespaces`, and commit the result.

This project is available under the [MIT License](LICENSE).
