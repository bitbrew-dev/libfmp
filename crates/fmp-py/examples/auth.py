"""Authentication: where the credential comes from and where it goes on the wire.

Precedence: an explicit ``token=`` always wins, otherwise ``FMP_API_KEY`` is read
(unset, empty, or whitespace-only counts as absent), and ``auth_mode="none"``
ignores the variable. Run with ``uv run auth.py``; the live calls need
``FMP_API_KEY``. Every other client here uses a dummy token and sends nothing.
"""

# standard library
import os

# fmp library
from fmp import FmpClient, FmpConfigError

# local
from _local_server import serve  # isort: skip

QUOTE_BODY = b'[{"symbol": "AAPL", "price": 1.0, "change": 0.0, "volume": 1.0}]'


def live_modes() -> None:
    """Calls FMP with the key in the header (the default) and in the query string."""
    from_env = FmpClient()
    explicit = FmpClient(token=os.environ["FMP_API_KEY"])
    # fmp_query appends ?apikey=... to the URL; error bodies redact it as [REDACTED].
    in_query = FmpClient(auth_mode="fmp_query")
    for name, client in [("env", from_env), ("token=", explicit), ("fmp_query", in_query)]:
        print(f"{name:>10}: {client.quote.short('AAPL')[0].price}")


def every_mode() -> None:
    """Builds one client per ``auth_mode`` value against a proxy; nothing is sent."""
    proxy = "https://proxy.example"
    dummy = "dummy-token"
    FmpClient(base_url=proxy, auth_mode="none")
    FmpClient(base_url=proxy, auth_mode="fmp_header", token=dummy)
    FmpClient(base_url=proxy, auth_mode="fmp_query", token=dummy)
    FmpClient(base_url=proxy, auth_mode="bearer", token=dummy)
    # The custom modes need auth_name: the header or query parameter to fill.
    FmpClient(base_url=proxy, auth_mode="custom_header", auth_name="X-Proxy-Token", auth_prefix="Bearer ", token=dummy)
    FmpClient(base_url=proxy, auth_mode="custom_query", auth_name="key", token=dummy)
    print("built one client per auth_mode")


def no_auth_router() -> None:
    """Talks to a credential-free router (a loopback server here) with no auth at all."""
    with serve(QUOTE_BODY) as base_url:
        # auth_mode="none" matters: without it an exported FMP_API_KEY would be sent
        # to this router. path_prefix="" drops the default "stable/" path segment.
        client = FmpClient(base_url=base_url, path_prefix="", auth_mode="none")
        print(f"router quote: {client.quote.short('AAPL')[0].price}")


def insecure_http() -> None:
    """Shows why plain-HTTP authentication needs an explicit, conspicuous opt-in."""
    try:
        FmpClient(base_url="http://proxy.example", token="dummy-token")
    except FmpConfigError as error:
        print(f"refused: {error}")
    # Anyone on the path can read a credential sent over plain HTTP. Opt in only for a
    # trusted network hop, such as a sidecar on a private interface. Loopback HTTP
    # (127.0.0.1, localhost) never needs the flag.
    FmpClient(base_url="http://proxy.example", token="dummy-token", danger_allow_insecure_authentication=True)
    print("built with danger_allow_insecure_authentication=True")


def missing_key() -> None:
    """Without a token or FMP_API_KEY, the default host refuses to build a client."""
    saved = os.environ.pop("FMP_API_KEY", None)
    try:
        FmpClient()
    except FmpConfigError as error:
        print(f"missing key: {error}")
    finally:
        if saved is not None:
            os.environ["FMP_API_KEY"] = saved


def main() -> None:
    """Runs every auth scenario."""
    live_modes()
    every_mode()
    no_auth_router()
    insecure_http()
    missing_key()


if __name__ == "__main__":
    main()
