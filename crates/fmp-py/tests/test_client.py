import json
import threading
from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from urllib.parse import parse_qs, urlsplit


SINGLE_QUOTE = [
    {"symbol": "AAPL", "price": 331.85501, "change": -6.33498, "volume": 28718014}
]
MULTIPLE_QUOTES = [
    {"symbol": "000001.SZ", "price": 14.02, "change": 0.12, "volume": 4294967296},
    {"symbol": "^VIX", "price": 15.25, "change": -0.5, "volume": 0},
]


@contextmanager
def fixture_server():
    requests = []

    class Handler(BaseHTTPRequestHandler):
        protocol_version = "HTTP/1.1"

        def do_GET(self):
            requests.append(
                {
                    "connection": self.client_address,
                    "path": self.path,
                    "headers": {name.lower(): value for name, value in self.headers.items()},
                }
            )
            symbol = parse_qs(urlsplit(self.path).query).get("symbol", [""])[0]
            if symbol == "EMPTY":
                status, body, content_type = 200, [], "application/json"
            elif symbol == "MULTI":
                status, body, content_type = 200, MULTIPLE_QUOTES, "application/json"
            elif symbol == "DENIED":
                status, body, content_type = 401, {"error": "denied"}, "application/json"
            elif symbol == "SECRET_DENIED":
                status, body, content_type = (
                    401,
                    b"denied?apikey=query-secret",
                    "text/plain",
                )
            elif symbol == "INVALID_JSON":
                status, body, content_type = 200, b"not-json", "application/json"
            else:
                status, body, content_type = 200, SINGLE_QUOTE, "application/json"

            payload = body if isinstance(body, bytes) else json.dumps(body).encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", content_type)
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)

        def log_message(self, _format, *_args):
            pass

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}/gateway", requests
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)


def test_direct_and_proxy_construction_and_call_shape():
    from fmp import FmpClient
    from fmp import _native
    from fmp.client import FmpClient as DomainFmpClient

    direct = FmpClient(token="direct-secret")
    assert type(direct).__name__ == "FmpClient"
    assert FmpClient is DomainFmpClient
    assert FmpClient.__module__ == "fmp.client"
    assert not hasattr(_native, "FmpClient")

    with fixture_server() as (base_url, requests):
        proxy = FmpClient(
            base_url=base_url,
            path_prefix="router/stable",
            auth_mode="none",
            headers={"X-Tenant": "blue", "X-Mode": "first", "x-mode": "second"},
            timeout=5.0,
            connect_timeout=2.0,
            follow_redirects=False,
        )
        rows = proxy.quote_short("AAPL")

    assert isinstance(rows, list)
    assert len(rows) == 1
    assert rows[0].symbol == "AAPL"
    assert rows[0].price == 331.85501
    assert rows[0].change == -6.33498
    assert rows[0].volume == 28718014
    try:
        rows[0].price = 0.0
    except AttributeError:
        pass
    else:
        raise AssertionError("QuoteShort attributes must remain read-only")
    assert requests[0]["path"] == "/gateway/router/stable/quote-short?symbol=AAPL"
    assert requests[0]["headers"]["x-tenant"] == "blue"
    assert requests[0]["headers"]["x-mode"] == "second"

    try:
        FmpClient("positional-is-not-supported")
    except TypeError:
        pass
    else:
        raise AssertionError("FmpClient configuration must remain keyword-only")


def test_empty_and_multiple_arrays_remain_lists_in_provider_order():
    import pickle
    import sys
    from types import ModuleType

    import fmp.quote
    from fmp import FmpClient, QuoteShort
    from fmp import _native

    with fixture_server() as (base_url, _requests):
        client = FmpClient(base_url=base_url, path_prefix="", auth_mode="none")
        empty = client.quote_short("EMPTY")
        multiple = client.quote_short("MULTI")

    assert empty == []
    assert isinstance(multiple, list)
    assert all(isinstance(row, QuoteShort) for row in multiple)
    assert QuoteShort is fmp.quote.QuoteShort
    assert isinstance(fmp.quote, ModuleType)
    assert fmp.quote is sys.modules["fmp.quote"]
    assert not hasattr(_native, "QuoteShort")
    assert fmp.quote.__file__ is not None
    assert QuoteShort.__module__ == "fmp.quote"
    assert [row.symbol for row in multiple] == ["000001.SZ", "^VIX"]
    assert multiple[0].volume == 4294967296
    restored = pickle.loads(pickle.dumps(multiple[0]))
    assert isinstance(restored, QuoteShort)
    assert (restored.symbol, restored.price, restored.change, restored.volume) == (
        multiple[0].symbol,
        multiple[0].price,
        multiple[0].change,
        multiple[0].volume,
    )


def test_sequential_calls_reuse_one_http_connection():
    from fmp import FmpClient

    with fixture_server() as (base_url, requests):
        client = FmpClient(base_url=base_url, path_prefix="", auth_mode="none")
        for _ in range(10):
            assert client.quote_short("AAPL")[0].symbol == "AAPL"

        assert len(requests) == 10
        assert len({request["connection"] for request in requests}) == 1


def test_auth_modes_and_default_headers_reach_the_same_endpoint():
    from fmp import FmpClient

    configurations = [
        ({"auth_mode": "none"}, None, None),
        (
            {"token": "inferred-header-secret"},
            ("apikey", "inferred-header-secret"),
            None,
        ),
        (
            {"auth_mode": "fmp_header", "token": "header-secret"},
            ("apikey", "header-secret"),
            None,
        ),
        (
            {"auth_mode": "fmp_query", "token": "query-secret"},
            None,
            ("apikey", "query-secret"),
        ),
        (
            {"auth_mode": "bearer", "token": "bearer-secret"},
            ("authorization", "Bearer bearer-secret"),
            None,
        ),
        (
            {
                "auth_mode": "custom_header",
                "auth_name": "X-Router-Token",
                "auth_prefix": "Token ",
                "token": "proxy-secret",
            },
            ("x-router-token", "Token proxy-secret"),
            None,
        ),
        (
            {
                "auth_mode": "custom_header",
                "auth_name": "X-Router-Key",
                "token": "unprefixed-secret",
            },
            ("x-router-key", "unprefixed-secret"),
            None,
        ),
        (
            {
                "auth_mode": "custom_query",
                "auth_name": "router_token",
                "token": "query-proxy-secret",
            },
            None,
            ("router_token", "query-proxy-secret"),
        ),
    ]

    with fixture_server() as (base_url, requests):
        for configuration, _expected_header, _expected_query in configurations:
            client = FmpClient(
                base_url=base_url,
                path_prefix="stable",
                headers={"X-Shared": "yes"},
                **configuration,
            )
            assert len(client.quote_short("AAPL")) == 1

    for request, (_configuration, expected_header, expected_query) in zip(
        requests, configurations
    ):
        assert request["headers"]["x-shared"] == "yes"
        if expected_header is not None:
            name, value = expected_header
            assert request["headers"][name] == value
        if expected_query is not None:
            name, value = expected_query
            assert parse_qs(urlsplit(request["path"]).query)[name] == [value]


def test_invalid_configuration_and_call_errors_are_structured():
    from fmp import (
        FmpClient,
        FmpConfigError,
        FmpDecodeError,
        FmpStatusError,
        FmpTransportError,
        FmpValidationError,
    )

    invalid_configurations = [
        {},
        {"auth_mode": "unknown"},
        {"auth_mode": "bearer"},
        {"auth_mode": "none", "token": "conflict"},
        {"auth_mode": "custom_header", "token": "missing-name"},
        {
            "auth_mode": "custom_query",
            "auth_name": "key",
            "auth_prefix": "Token ",
            "token": "secret",
        },
        {"base_url": "https://proxy.example", "timeout": 0.0},
        {"base_url": "https://proxy.example", "timeout": float("nan")},
        {"base_url": "https://proxy.example", "connect_timeout": 1e300},
        {"base_url": "https://proxy.example", "headers": {"bad name": "value"}},
    ]
    for configuration in invalid_configurations:
        try:
            FmpClient(**configuration)
        except FmpConfigError as error:
            assert error.category == "configuration"
            assert error.endpoint is None
            assert error.status is None
            assert error.body is None
        else:
            raise AssertionError(f"configuration unexpectedly succeeded: {configuration!r}")

    with fixture_server() as (base_url, _requests):
        client = FmpClient(base_url=base_url, path_prefix="", auth_mode="none")

        for symbol in ("", "BAD,SYMBOL"):
            try:
                client.quote_short(symbol)
            except FmpValidationError as error:
                assert error.category == "validation"
                assert error.endpoint is None
            else:
                raise AssertionError(f"symbol unexpectedly succeeded: {symbol!r}")

        try:
            client.quote_short("DENIED")
        except FmpStatusError as error:
            assert error.category == "status"
            assert error.endpoint == "quote-short"
            assert error.status == 401
            assert error.body == '{"error": "denied"}'
            assert error.body_truncated is False
        else:
            raise AssertionError("non-success response did not raise FmpStatusError")

        try:
            client.quote_short("INVALID_JSON")
        except FmpDecodeError as error:
            assert error.category == "decode"
            assert error.endpoint == "quote-short"
            assert error.status == 200
            assert error.body == "not-json"
            assert error.body_truncated is False
        else:
            raise AssertionError("invalid JSON did not raise FmpDecodeError")

    transport_client = FmpClient(
        base_url="http://127.0.0.1:0",
        path_prefix="",
        auth_mode="none",
        timeout=1.0,
        connect_timeout=0.25,
    )
    try:
        transport_client.quote_short("AAPL")
    except FmpTransportError as error:
        assert error.category == "transport"
        assert error.endpoint == "quote-short"
        assert error.status is None
        assert error.body is None
        assert error.body_truncated is None
    else:
        raise AssertionError("connection failure did not raise FmpTransportError")


def test_status_error_redacts_configured_query_secret():
    from fmp import FmpClient, FmpStatusError

    with fixture_server() as (base_url, _requests):
        client = FmpClient(
            token="query-secret",
            base_url=base_url,
            path_prefix="",
            auth_mode="fmp_query",
        )
        try:
            client.quote_short("SECRET_DENIED")
        except FmpStatusError as error:
            assert error.category == "status"
            assert error.endpoint == "quote-short"
            assert error.status == 401
            assert error.body == "denied?apikey=[REDACTED]"
            assert error.body_truncated is False
            diagnostic = f"{error!s} {error!r} {error.body}"
            assert "query-secret" not in diagnostic
        else:
            raise AssertionError("non-success response did not raise FmpStatusError")
