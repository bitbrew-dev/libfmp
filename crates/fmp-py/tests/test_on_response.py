"""Issue #417: ``FmpClient(on_response=...)`` sees allowlisted headers on success."""

import threading
from concurrent.futures import ThreadPoolExecutor
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer

QUOTE_SHORT_ROW = {"symbol": "AAPL", "price": 232.8, "change": 2.1, "volume": 44000000}

VALET_HEADERS = (
    ("X-Proxy-Cache", "HIT"),
    ("Set-Cookie", "session=cookie-secret"),
    ("Authorization", "Bearer header-secret"),
    ("X-Unrelated", "unrelated-value"),
    ("X-Proxy-Daily-Remaining", "42"),
)

Call = tuple[str, int, dict[str, str]]


def observed(fixture_server: FixtureServer) -> tuple[Any, list[Call]]:
    """A client against ``fixture_server`` whose callback records each call."""
    from fmp import FmpClient

    calls: list[Call] = []

    def on_response(endpoint_id: str, status: int, headers: dict[str, str]) -> None:
        calls.append((endpoint_id, status, headers))

    client = FmpClient(base_url=fixture_server.base_url, path_prefix="", auth_mode="none", on_response=on_response)
    return client, calls


def test_on_response_receives_endpoint_status_and_allowlisted_headers(fixture_server: FixtureServer) -> None:
    """A valet success reports its cache status and remaining budget, nothing else."""
    fixture_server.route("/quote-short", [QUOTE_SHORT_ROW], headers=VALET_HEADERS)
    client, calls = observed(fixture_server)

    rows = client.quote.short("AAPL")

    assert [row.symbol for row in rows] == ["AAPL"]
    assert calls == [("quote-short", 200, {"x-proxy-cache": "HIT", "x-proxy-daily-remaining": "42"})]
    for secret in ("cookie-secret", "header-secret", "unrelated-value"):
        assert secret not in repr(calls)


def test_on_response_gets_an_empty_dict_and_first_duplicate(fixture_server: FixtureServer) -> None:
    """No allowlisted header gives ``{}``; a repeated name keeps its first value."""
    fixture_server.route("/quote-short", [QUOTE_SHORT_ROW], headers=(("X-Unrelated", "x"),))
    fixture_server.route("/quote", [], headers=(("X-Proxy-Cache", "MISS"), ("X-Proxy-Cache", "HIT")))
    client, calls = observed(fixture_server)

    client.quote.short("AAPL")
    client.quote.full("AAPL")

    assert calls == [("quote-short", 200, {}), ("quote", 200, {"x-proxy-cache": "MISS"})]


def test_on_response_is_not_called_on_errors(fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """Status and provider-message errors keep their headers on the exception only."""
    fixture_server.route("/quote-short", b"slow down", status=429, content_type="text/plain", headers=VALET_HEADERS)
    fixture_server.route("/quote", b"Invalid name", headers=VALET_HEADERS)
    client, calls = observed(fixture_server)

    with pytest.raises(errors.FmpStatusError) as raised:
        client.quote.short("AAPL")
    assert raised.value.headers == {"x-proxy-cache": "HIT", "x-proxy-daily-remaining": "42"}
    with pytest.raises(errors.FmpError):
        client.quote.full("AAPL")

    assert calls == []


def test_on_response_exception_propagates_to_the_caller(fixture_server: FixtureServer) -> None:
    """An exception raised by the callback replaces the method's return value."""
    from fmp import FmpClient

    fixture_server.route("/quote-short", [QUOTE_SHORT_ROW], headers=VALET_HEADERS)

    class BudgetExhausted(Exception):
        pass

    def on_response(_endpoint_id: str, _status: int, headers: dict[str, str]) -> None:
        raise BudgetExhausted(headers["x-proxy-daily-remaining"])

    client = FmpClient(base_url=fixture_server.base_url, path_prefix="", auth_mode="none", on_response=on_response)
    with pytest.raises(BudgetExhausted, match="^42$"):
        client.quote.short("AAPL")


def test_on_response_threads_see_only_their_own_headers(fixture_server: FixtureServer) -> None:
    """Concurrent calls each get the headers of their own response, on their own thread."""
    symbols = [f"S{index}" for index in range(16)]
    for symbol in symbols:
        fixture_server.route(
            f"/quote-short?symbol={symbol}",
            [dict(QUOTE_SHORT_ROW, symbol=symbol)],
            headers=(("X-Proxy-Daily-Remaining", symbol),),
        )
    from fmp import FmpClient

    seen: dict[int, list[str]] = {}

    def on_response(_endpoint_id: str, _status: int, headers: dict[str, str]) -> None:
        seen.setdefault(threading.get_ident(), []).append(headers["x-proxy-daily-remaining"])

    client = FmpClient(base_url=fixture_server.base_url, path_prefix="", auth_mode="none", on_response=on_response)

    def call(symbol: str) -> tuple[int, str]:
        rows = client.quote.short(symbol)
        assert [row.symbol for row in rows] == [symbol]
        return threading.get_ident(), seen[threading.get_ident()][-1]

    with ThreadPoolExecutor(max_workers=8) as pool:
        results = list(pool.map(call, symbols))

    assert [remaining for _, remaining in results] == symbols
    assert sorted(value for values in seen.values() for value in values) == sorted(symbols)


def test_on_response_must_be_callable(fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-callable ``on_response`` is a configuration error; ``None`` is accepted."""
    from fmp import FmpClient

    with pytest.raises(errors.FmpConfigError, match="on_response must be callable"):
        FmpClient(base_url=fixture_server.base_url, path_prefix="", auth_mode="none", on_response=42)
    FmpClient(base_url=fixture_server.base_url, path_prefix="", auth_mode="none", on_response=None)
