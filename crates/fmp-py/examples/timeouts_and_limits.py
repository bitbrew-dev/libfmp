"""Timeouts and response-size limits, including the separate bulk defaults.

Defaults when the argument is omitted:

| Setting                     | ``client.bulk`` methods | every other method |
| --------------------------- | ----------------------- | ------------------ |
| ``timeout``                 | 600 s                   | 30 s               |
| ``max_response_body_bytes`` | 256 MiB                 | 64 MiB             |

An explicit value replaces BOTH defaults: ``timeout=10`` also caps bulk downloads
at 10 s, which is far too short for a multi-megabyte bulk body. Prefer separate
clients for bulk and for regular calls when you tune these. A timeout and an
oversized body both raise ``FmpTransportError``. Staged against a
loopback server; run with ``uv run timeouts_and_limits.py`` (no key needed).
"""

# fmp library
from fmp import FmpClient, FmpError

# local
from _local_server import serve  # isort: skip

QUOTE_BODY = b'[{"symbol": "AAPL", "price": 1.0, "change": 0.0, "volume": 1.0}]'
CSV_BODY = b"symbol\n"


def attempt(label: str, client: FmpClient, *, bulk: bool = False) -> None:
    """Makes one call and reports whether it succeeded or which error it raised."""
    try:
        rows = client.bulk.company_profiles("0") if bulk else client.quote.short("AAPL")
        print(f"{label}: ok, {len(rows)} rows")
    except FmpError as error:
        print(f"{label}: {type(error).__name__}: {error}")


def main() -> None:
    """Shows each timeout and size-limit rule against a slow or large response."""
    # timeout bounds the whole request; connect_timeout only the TCP/TLS connect.
    # Both are positive finite seconds; 0, negative, NaN or inf raise FmpConfigError.
    FmpClient(timeout=10.0, connect_timeout=2.0)

    with serve(QUOTE_BODY, delay=1.0) as base_url:
        slow = FmpClient(base_url=base_url, path_prefix="", auth_mode="none", timeout=0.3)
        attempt("regular call, timeout=0.3, server takes 1 s", slow)

    with serve(CSV_BODY, content_type="text/csv", delay=1.0) as base_url:
        default = FmpClient(base_url=base_url, path_prefix="", auth_mode="none")
        attempt("bulk call, default timeout (600 s)", default, bulk=True)
        tuned = FmpClient(base_url=base_url, path_prefix="", auth_mode="none", timeout=0.3)
        attempt("bulk call, timeout=0.3 also applies to bulk", tuned, bulk=True)

    large = b"[" + b", ".join([QUOTE_BODY[1:-1]] * 64) + b"]"
    with serve(large) as base_url:
        default = FmpClient(base_url=base_url, path_prefix="", auth_mode="none")
        attempt(f"{len(large)}-byte body, default limit (64 MiB)", default)
        capped = FmpClient(base_url=base_url, path_prefix="", auth_mode="none", max_response_body_bytes=1024)
        attempt(f"{len(large)}-byte body, max_response_body_bytes=1024", capped)

    big_csv = CSV_BODY * 1024
    with serve(big_csv, content_type="text/csv") as base_url:
        capped = FmpClient(base_url=base_url, path_prefix="", auth_mode="none", max_response_body_bytes=1024)
        attempt(f"{len(big_csv)}-byte bulk body, max_response_body_bytes=1024 applies to bulk too", capped, bulk=True)


if __name__ == "__main__":
    main()
