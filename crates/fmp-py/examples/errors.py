"""Errors: one exception class per failure stage, each with structured attributes.

Every class derives from ``fmp.FmpError`` and carries ``category``, ``endpoint``,
``status``, ``body``, ``body_truncated``, ``decode_path`` and ``decode_kind``.
The failures are staged against a loopback server so each one is reproducible;
run with ``uv run errors.py`` (only the bad-key case touches the network).
"""

# fmp library
from fmp import (
    FmpClient,
    FmpConfigError,
    FmpDecodeError,
    FmpError,
    FmpStatusError,
    FmpTransportError,
    FmpValidationError,
)

# local
from _local_server import serve  # isort: skip


def local_client(base_url: str) -> FmpClient:
    """Builds a credential-free client for the loopback server."""
    return FmpClient(base_url=base_url, path_prefix="", auth_mode="none", timeout=5.0)


def show(error: FmpError) -> None:
    """Prints the structured attributes every fmp error carries."""
    print(f"  {type(error).__name__}: {error}")
    print(f"  category={error.category} endpoint={error.endpoint} status={error.status}")
    if error.body is not None:
        print(f"  body={error.body!r} truncated={error.body_truncated}")
    if error.decode_path is not None:
        print(f"  decode_path={error.decode_path} decode_kind={error.decode_kind}")


def main() -> None:
    """Triggers each failure stage in order."""
    print("config: the client cannot be built")
    try:
        FmpClient(base_url="https://proxy.example", timeout=0.0)
    except FmpConfigError as error:
        show(error)

    print("validation: an argument is rejected locally, nothing is sent")
    try:
        FmpClient(base_url="https://proxy.example", auth_mode="none").quote.short("")
    except FmpValidationError as error:
        show(error)

    print("transport: no response at all (nothing listens on port 1)")
    try:
        FmpClient(base_url="http://127.0.0.1:1", path_prefix="", auth_mode="none", connect_timeout=1.0).quote.short(
            "AAPL"
        )
    except FmpTransportError as error:
        show(error)

    print("status: a non-success HTTP status")
    with serve(b'{"Error Message": "Invalid API KEY."}', status=401) as base_url:
        try:
            local_client(base_url).quote.short("AAPL")
        except FmpStatusError as error:
            show(error)

    print("status: the real provider rejecting a bad key (needs network)")
    try:
        FmpClient(token="not-a-real-key").quote.short("AAPL")
    except FmpStatusError as error:
        show(error)

    # FMP sometimes answers 200 OK with its own error message instead of data. The
    # client recognizes that shape and raises FmpStatusError, not a decode error.
    print("status: a provider error message inside a 200 body")
    with serve(b'{"Error Message": "Limit Reach."}') as base_url:
        try:
            local_client(base_url).quote.short("AAPL")
        except FmpStatusError as error:
            show(error)

    # decode_path names the failing member ([1].price: second row, price field) and
    # decode_kind the reason. Neither carries the offending value, so both are safe
    # to log. body is a size-bounded excerpt of the raw response (an echoed apikey is
    # redacted), so it can contain response values.
    print("decode: the body does not match the documented shape")
    body = (
        b'[{"symbol": "AAPL", "price": 1.0, "change": 0.0, "volume": 1.0},'
        b' {"symbol": "MSFT", "price": "n/a", "change": 0.0, "volume": 1.0}]'
    )
    with serve(body) as base_url:
        try:
            local_client(base_url).quote.short("AAPL")
        except FmpDecodeError as error:
            show(error)

    print("catch-all: FmpError is the base of every class above")
    try:
        FmpClient(base_url="https://proxy.example", auth_mode="none").quote.short("")
    except FmpError as error:
        print(f"  caught {type(error).__name__} via FmpError")


if __name__ == "__main__":
    main()
