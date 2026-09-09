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
from fmp import FmpClient

client = FmpClient()
rows = client.quote_short("AAPL")
print(rows)
```

`FmpClient()` reads the `FMP_API_KEY` environment variable when `token` is
omitted (unset, empty, or whitespace-only counts as absent); an explicit
`token=...` always wins, and `auth_mode="none"` ignores the variable. With
neither a token nor the variable, the default host raises `FmpConfigError`
naming `FMP_API_KEY`, while a custom `base_url` selects no auth.

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
trusted because its constructor could not be seen. Pass a directory argument
to check a different registry tree.

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
`BinaryPayload` instead of a list of models: `data` is the body as `bytes`,
alongside `content_type`, `content_disposition`, and `byte_len`.
`BinaryPayload` is hand-written in `src/binary.rs` and exported from the
package root as `fmp.BinaryPayload`.

The same run emits the wiring that binds the generated code into the
extension, so adding a domain never edits `lib.rs` or `client.rs`:

| File | Contents |
|------|----------|
| `src/namespaces/mod.rs` | one `mod` declaration per domain |
| `src/registration.rs` | `register_namespaces`: every `fmp._native.<path>` submodule with its models (read back from `src/models/**` with `syn`) and namespace classes, published in `sys.modules` |
| `src/client_namespaces.rs` | one `FmpClient` getter per domain, a second `#[pymethods]` block (pyo3's `multiple-pymethods` feature) |
| `src/facade_domains.rs` | a wildcard `reexport_module_members!` per `fmp._native.<path>`, which is what writes the public `python/fmp/<path>/__init__.py` packages |

The hand-written `src/facade.rs` keeps only the top-level `fmp` surface.

## Regenerating the stubs and public packages

The `.pyi` stubs under `python/fmp/_native/` and the public
`python/fmp/<path>/__init__.py` packages are written by `pyo3-stub-gen`
through the `stub_gen` binary, from the repository root:

```console
cargo run -p fmp-py --bin stub_gen
```

`stub_gen` post-processes each `__init__.pyi` (absolute imports, no
`# ruff: noqa` header, `ruff format --isolated --line-length 88` in `.py`
mode) so a run on an unchanged tree leaves `git status` clean. It needs
`ruff` at the version pinned in `.pre-commit-config.yaml` (currently
0.15.12): either that exact `ruff` on `PATH` or `uvx`, which fetches it. Run
it after `gen_models` or `gen_namespaces`, and commit the result.

This project is available under the [MIT License](LICENSE).
