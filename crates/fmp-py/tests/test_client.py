"""Runtime contract of ``FmpClient`` and the ``client.quote`` namespace."""

import pickle
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp import FmpClient
from fmp.quote import Quote, QuoteShort

QUOTE_SHORT_PATH = "/quote-short"
QUOTE_FULL_PATH = "/quote"
MUTUAL_FUNDS_PATH = "/batch-mutualfund-quotes"


def test_constructor_is_keyword_only() -> None:
    """Positional configuration is rejected; the keyword form builds a client."""
    with pytest.raises(TypeError):
        FmpClient("positional-is-not-supported")
    assert type(FmpClient(token="direct-secret")).__name__ == "FmpClient"


def test_proxy_configuration_shapes_the_request(fixture_server: FixtureServer) -> None:
    """Base URL, path prefix, and default headers all reach the wire."""
    fixture_server.route("/gateway/router/stable/quote-short", load_fixture("quote_short.json"))
    proxy = FmpClient(
        base_url=f"{fixture_server.base_url}/gateway",
        path_prefix="router/stable",
        auth_mode="none",
        headers={"X-Tenant": "blue", "X-Mode": "first", "x-mode": "second"},
        timeout=5.0,
        connect_timeout=2.0,
        follow_redirects=False,
    )
    rows = proxy.quote.short("AAPL")

    assert [row.symbol for row in rows] == ["AAPL"]
    request = fixture_server.requests[0]
    assert request.target == "/gateway/router/stable/quote-short?symbol=AAPL"
    assert request.headers["x-tenant"] == "blue"
    assert request.headers["x-mode"] == "second"


def test_quote_full_decodes_documented_fixture(client: Any, fixture_server: FixtureServer) -> None:
    """``quote.full`` maps the documented wire fixture to ``Quote`` exactly."""
    fixture_server.route(QUOTE_FULL_PATH, load_fixture("quote.json"))
    rows = client.quote.full("AAPL")

    assert fixture_server.requests[0].target == "/quote?symbol=AAPL"
    assert len(rows) == 1
    quote = rows[0]
    assert isinstance(quote, Quote)
    assert quote.symbol == "AAPL"
    assert quote.name == "Apple Inc."
    assert quote.price == 331.85501
    assert quote.change_percentage == -1.8732
    assert quote.change == -6.33498
    assert quote.volume == 28_718_014
    assert quote.day_low == 329.59
    assert quote.day_high == 334.48
    assert quote.year_high == 344.57
    assert quote.year_low == 201.5
    assert quote.market_cap == 4_874_072_686_740
    assert quote.price_avg_50 == 308.5888
    assert quote.price_avg_200 == 277.21344
    assert quote.exchange == "NASDAQ"
    assert quote.open == 333.13
    assert quote.previous_close == 338.18999
    assert quote.timestamp == 1_785_430_812


def test_quote_short_rows_are_frozen_and_picklable(client: Any, fixture_server: FixtureServer) -> None:
    """Empty and multi-row bodies stay lists in provider order; rows are immutable values."""
    fixture_server.route(f"{QUOTE_SHORT_PATH}?symbol=EMPTY", load_fixture("quote_short_empty.json"))
    fixture_server.route(f"{QUOTE_SHORT_PATH}?symbol=MULTI", load_fixture("quote_short_multiple.json"))

    assert client.quote.short("EMPTY") == []
    multiple = client.quote.short("MULTI")

    assert all(isinstance(row, QuoteShort) for row in multiple)
    assert [row.symbol for row in multiple] == ["000001.SZ", "^VIX"]
    assert multiple[0].volume == 4_294_967_296
    with pytest.raises(AttributeError):
        multiple[0].price = 0.0
    restored = pickle.loads(pickle.dumps(multiple[0]))
    assert isinstance(restored, QuoteShort)
    assert (restored.symbol, restored.price, restored.change, restored.volume) == (
        multiple[0].symbol,
        multiple[0].price,
        multiple[0].change,
        multiple[0].volume,
    )


def test_mutual_funds_uses_the_batch_path_with_the_short_flag(client: Any, fixture_server: FixtureServer) -> None:
    """``quote.mutual_funds`` takes no arguments; libfmp adds the compact-quote flag."""
    fixture_server.route(MUTUAL_FUNDS_PATH, load_fixture("quote_short_multiple.json"))
    rows = client.quote.mutual_funds()

    assert fixture_server.requests[0].path == MUTUAL_FUNDS_PATH
    assert fixture_server.requests[0].query == {"short": ["true"]}
    assert [row.symbol for row in rows] == ["000001.SZ", "^VIX"]


def test_sequential_calls_reuse_one_http_connection(client: Any, fixture_server: FixtureServer) -> None:
    """Repeated calls on one client share a keep-alive connection."""
    fixture_server.route(QUOTE_SHORT_PATH, load_fixture("quote_short.json"))
    for _ in range(10):
        assert client.quote.short("AAPL")[0].symbol == "AAPL"

    assert len(fixture_server.requests) == 10
    assert len({request.connection for request in fixture_server.requests}) == 1


AUTH_CONFIGURATIONS = [
    pytest.param({"auth_mode": "none"}, None, None, id="none"),
    pytest.param({"token": "inferred-header-secret"}, ("apikey", "inferred-header-secret"), None, id="inferred"),
    pytest.param({"auth_mode": "fmp_header", "token": "header-secret"}, ("apikey", "header-secret"), None, id="header"),
    pytest.param({"auth_mode": "fmp_query", "token": "query-secret"}, None, ("apikey", "query-secret"), id="query"),
    pytest.param(
        {"auth_mode": "bearer", "token": "bearer-secret"}, ("authorization", "Bearer bearer-secret"), None, id="bearer"
    ),
    pytest.param(
        {"auth_mode": "custom_header", "auth_name": "X-Router-Token", "auth_prefix": "Token ", "token": "proxy-secret"},
        ("x-router-token", "Token proxy-secret"),
        None,
        id="custom-header-prefixed",
    ),
    pytest.param(
        {"auth_mode": "custom_header", "auth_name": "X-Router-Key", "token": "unprefixed-secret"},
        ("x-router-key", "unprefixed-secret"),
        None,
        id="custom-header",
    ),
    pytest.param(
        {"auth_mode": "custom_query", "auth_name": "router_token", "token": "query-proxy-secret"},
        None,
        ("router_token", "query-proxy-secret"),
        id="custom-query",
    ),
]


@pytest.mark.parametrize(("configuration", "expected_header", "expected_query"), AUTH_CONFIGURATIONS)
def test_auth_modes_and_default_headers_reach_the_same_endpoint(
    fixture_server: FixtureServer,
    configuration: dict[str, str],
    expected_header: tuple[str, str] | None,
    expected_query: tuple[str, str] | None,
) -> None:
    """Every authentication mode sends its credential alongside the shared headers."""
    fixture_server.route("/stable/quote-short", load_fixture("quote_short.json"))
    client = FmpClient(
        base_url=fixture_server.base_url, path_prefix="stable", headers={"X-Shared": "yes"}, **configuration
    )

    assert len(client.quote.short("AAPL")) == 1
    request = fixture_server.requests[0]
    assert request.headers["x-shared"] == "yes"
    if expected_header is not None:
        name, value = expected_header
        assert request.headers[name] == value
    if expected_query is not None:
        name, value = expected_query
        assert request.query[name] == [value]


INVALID_CONFIGURATIONS = [
    pytest.param({}, id="no-auth-no-base-url"),
    pytest.param({"auth_mode": "unknown"}, id="unknown-mode"),
    pytest.param({"auth_mode": "bearer"}, id="bearer-without-token"),
    pytest.param({"auth_mode": "none", "token": "conflict"}, id="none-with-token"),
    pytest.param({"auth_mode": "custom_header", "token": "missing-name"}, id="custom-header-without-name"),
    pytest.param(
        {"auth_mode": "custom_query", "auth_name": "key", "auth_prefix": "Token ", "token": "secret"},
        id="custom-query-with-prefix",
    ),
    pytest.param({"base_url": "https://proxy.example", "timeout": 0.0}, id="zero-timeout"),
    pytest.param({"base_url": "https://proxy.example", "timeout": float("nan")}, id="nan-timeout"),
    pytest.param({"base_url": "https://proxy.example", "connect_timeout": 1e300}, id="huge-connect-timeout"),
    pytest.param({"base_url": "https://proxy.example", "headers": {"bad name": "value"}}, id="bad-header-name"),
    pytest.param({"token": "secret", "base_url": "http://proxy.example"}, id="insecure-authenticated-http"),
]


@pytest.mark.parametrize("configuration", INVALID_CONFIGURATIONS)
def test_invalid_configuration_raises_config_error(errors: SimpleNamespace, configuration: dict[str, Any]) -> None:
    """Invalid construction raises ``FmpConfigError`` with no request context."""
    with pytest.raises(errors.FmpConfigError) as raised:
        FmpClient(**configuration)
    error = raised.value
    assert error.category == "configuration"
    assert (error.endpoint, error.status, error.body) == (None, None, None)


def test_insecure_authenticated_http_needs_explicit_opt_in() -> None:
    """The danger flag unlocks authenticated plain HTTP off loopback."""
    FmpClient(
        token="secret",
        base_url="http://proxy.example",
        max_response_body_bytes=1024,
        danger_allow_insecure_authentication=True,
    )


@pytest.mark.parametrize(
    ("symbol", "message"),
    [
        pytest.param("", "symbol: value must not be empty or whitespace-only", id="empty"),
        pytest.param("   ", "symbol: value must not be empty or whitespace-only", id="whitespace"),
        pytest.param("BAD,SYMBOL", "symbol: ticker must not contain a comma", id="comma"),
    ],
)
def test_validation_errors_name_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, symbol: str, message: str
) -> None:
    """Local validation fails before any request and names the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.quote.short(symbol)
    error = raised.value
    assert str(error) == message
    assert error.category == "validation"
    assert error.endpoint is None
    assert fixture_server.requests == []


def test_status_error_is_structured(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-success status surfaces the endpoint id, status, and body."""
    fixture_server.route(QUOTE_SHORT_PATH, {"error": "denied"}, status=401)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.quote.short("DENIED")
    error = raised.value
    assert error.category == "status"
    assert error.endpoint == "quote-short"
    assert error.status == 401
    assert error.body == '{"error": "denied"}'
    assert error.body_truncated is False


def test_decode_error_is_structured(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A 200 with an undecodable body raises ``FmpDecodeError``."""
    fixture_server.route(QUOTE_SHORT_PATH, b"not-json")
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.quote.short("INVALID_JSON")
    error = raised.value
    assert error.category == "decode"
    assert error.endpoint == "quote-short"
    assert error.status == 200
    assert error.body == "not-json"
    assert error.body_truncated is False


def test_transport_error_is_structured(errors: SimpleNamespace) -> None:
    """A connection failure raises ``FmpTransportError`` without a response."""
    client = FmpClient(
        base_url="http://127.0.0.1:0", path_prefix="", auth_mode="none", timeout=1.0, connect_timeout=0.25
    )
    with pytest.raises(errors.FmpTransportError) as raised:
        client.quote.short("AAPL")
    error = raised.value
    assert error.category == "transport"
    assert error.endpoint == "quote-short"
    assert (error.status, error.body, error.body_truncated) == (None, None, None)


def test_status_error_redacts_configured_query_secret(fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """The query credential never appears in a status error's body or repr."""
    fixture_server.route(QUOTE_SHORT_PATH, b"denied?apikey=query-secret", status=401, content_type="text/plain")
    client = FmpClient(token="query-secret", base_url=fixture_server.base_url, path_prefix="", auth_mode="fmp_query")
    with pytest.raises(errors.FmpStatusError) as raised:
        client.quote.short("SECRET_DENIED")
    error = raised.value
    assert error.status == 401
    assert error.body == "denied?apikey=[REDACTED]"
    assert error.body_truncated is False
    assert "query-secret" not in f"{error!s} {error!r} {error.body}"
