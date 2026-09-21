"""Opt-in live tests: direct provider authentication and a production proxy route.

Both tests skip unless ``FMP_LIVE_TESTS=1`` and the credentials are exported,
so the default ``pytest`` run never opens a socket::

    FMP_LIVE_TESTS=1 FMP_API_KEY=... pytest crates/fmp-py/tests/test_live_opt_in.py -rs

The values are captured at import time because the autouse
``isolated_api_key_env`` fixture in ``conftest.py`` removes ``FMP_API_KEY``
before every test; the client is therefore always built with an explicit
``token``. Proxy variables mirror the README router example:
``FMP_PROXY_BASE_URL`` and ``FMP_PROXY_TOKEN`` are required, the token is sent
as ``X-Proxy-Token: Bearer <token>``, ``FMP_PROXY_PATH_PREFIX`` overrides the
``router/stable`` prefix, and ``FMP_TENANT`` adds ``X-Tenant`` when set.
Secrets are never printed: on failure the assertions prove the exception text
does not contain them before re-raising.
"""

import os
from typing import Any

import pytest
from fmp import FmpClient


def _non_empty(name: str) -> str | None:
    value = os.environ.get(name, "").strip()
    return value or None


LIVE = _non_empty("FMP_LIVE_TESTS") == "1"
API_KEY = _non_empty("FMP_API_KEY")
PROXY_BASE_URL = _non_empty("FMP_PROXY_BASE_URL")
PROXY_TOKEN = _non_empty("FMP_PROXY_TOKEN")
PROXY_PATH_PREFIX = _non_empty("FMP_PROXY_PATH_PREFIX") or "router/stable"
TENANT = _non_empty("FMP_TENANT")

requires_direct_credentials = pytest.mark.skipif(
    not (LIVE and API_KEY),
    reason="set FMP_LIVE_TESTS=1 and FMP_API_KEY to run the direct live test",
)
requires_proxy_credentials = pytest.mark.skipif(
    not (LIVE and PROXY_BASE_URL and PROXY_TOKEN),
    reason="set FMP_LIVE_TESTS=1, FMP_PROXY_BASE_URL and FMP_PROXY_TOKEN to run the proxy live test",
)


def _quote_short_without_leaking(client: FmpClient, secret: str) -> list[Any]:
    try:
        return client.quote.short("AAPL")
    except Exception as error:
        diagnostic = f"{error!r} {error}"
        assert secret not in diagnostic, "exception text leaked the configured secret"
        raise


def _assert_apple_rows(rows: list[Any]) -> None:
    assert rows, "quote-short returned an empty array"
    assert rows[0].symbol == "AAPL"


@requires_direct_credentials
def test_direct_header_authentication_returns_quote_short_rows() -> None:
    """The default host with the ``apikey`` header returns AAPL rows and never echoes the key."""
    assert API_KEY is not None
    client = FmpClient(token=API_KEY)
    assert API_KEY not in repr(client)
    _assert_apple_rows(_quote_short_without_leaking(client, API_KEY))


@requires_proxy_credentials
def test_production_proxy_route_returns_quote_short_rows() -> None:
    """The README router configuration returns AAPL rows and never echoes the proxy token."""
    assert PROXY_BASE_URL is not None
    assert PROXY_TOKEN is not None
    client = FmpClient(
        base_url=PROXY_BASE_URL,
        path_prefix=PROXY_PATH_PREFIX,
        auth_mode="custom_header",
        auth_name="X-Proxy-Token",
        auth_prefix="Bearer ",
        token=PROXY_TOKEN,
        headers={"X-Tenant": TENANT} if TENANT else None,
    )
    assert PROXY_TOKEN not in repr(client)
    _assert_apple_rows(_quote_short_without_leaking(client, PROXY_TOKEN))
