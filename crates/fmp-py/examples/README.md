# fmp-py-sdk examples

Small, runnable scripts for the `fmp` Python client (`pip install fmp-py-sdk`).
Each one covers a single topic and is commented where the configuration is easy
to get wrong.

## Running

From this directory, with [uv](https://docs.astral.sh/uv/):

```console
export FMP_API_KEY=...   # or inject it from your secret manager
uv run quickstart.py
```

- `uv run` creates a local `.venv` with the released `fmp-py-sdk` from PyPI
  (Python 3.10 or newer); nothing in this repository is built.
- The client reads `FMP_API_KEY` from the environment. No script prints it,
  and no script takes it as an argument.
- Files a script writes (the bulk CSV, the XLSX report) go to a fresh temporary
  directory, whose path is printed.
- Scripts marked "offline" stage their scenarios against a loopback server in
  `_local_server.py`, so they need neither a key nor network access.

## Scripts

| Script | Shows | Needs |
|--------|-------|-------|
| `quickstart.py` | `FmpClient()` from the environment, `quote.short`, an income statement with `period` and `limit`, a DCF valuation | key |
| `auth.py` | env key vs `token=`, every `auth_mode` value, a credential-free router with `auth_mode="none"`, why plain-HTTP auth needs `danger_allow_insecure_authentication` | key |
| `timeouts_and_limits.py` | `timeout` and `connect_timeout`; bulk defaults (600 s, 256 MiB) vs the rest (30 s, 64 MiB); an explicit value applies to bulk too | offline |
| `errors.py` | config, validation, transport, status (including a provider error inside a 200 body) and decode errors; `decode_path` and `decode_kind` | offline, one call needs network |
| `nullable_fields.py` | `Optional` members such as `CompanyProfile.isin` / `country`; congressional net-worth columns that may be `None` and the `additional_columns` dict | key |
| `lifecycle_and_threads.py` | `with FmpClient() as client:`, `close()` semantics, a `ThreadPoolExecutor` fan-out on one shared client | key |
| `bulk.py` | one small bulk endpoint, its row count, and a CSV written with the `csv` module | key with bulk access |
| `binary_and_dicts.py` | `statements.reports.xlsx` bytes saved to a file; `to_dict()` for JSON or a DataFrame hand-off | key |
| `pagination.py` | a `page` / `limit` loop that stops on a short page | key |

## Type checking

The scripts pass `mypy --strict` against the stubs shipped in the wheel; the
binding's test suite (`tests/test_stubs.py`) checks this whenever `mypy` is
installed.
