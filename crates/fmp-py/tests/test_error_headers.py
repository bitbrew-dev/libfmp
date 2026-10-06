"""Issue #416: status errors keep only the allowlisted response headers."""

import time
from email.utils import formatdate
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer

QUOTE_SHORT_PATH = "/quote-short"

VALET_HEADERS = (
    ("Retry-After", "12"),
    ("X-Proxy-Key-Id", "vk_123"),
    ("X-Proxy-Error", "rate_limited"),
    ("Set-Cookie", "session=cookie-secret"),
    ("Authorization", "Bearer header-secret"),
    ("apikey", "query-secret"),
    ("X-Unrelated", "unrelated-value"),
    ("X-Proxy-Burst-Remaining", "0"),
    ("X-RateLimit-Remaining", "0"),
)

ALLOWLISTED = {
    "retry-after": "12",
    "x-proxy-key-id": "vk_123",
    "x-proxy-error": "rate_limited",
    "x-proxy-burst-remaining": "0",
    "x-ratelimit-remaining": "0",
}


def raise_status(client: Any, errors: SimpleNamespace) -> Any:
    with pytest.raises(errors.FmpStatusError) as raised:
        client.quote.short("AAPL")
    return raised.value


def test_rate_limited_status_error_exposes_retry_after_and_proxy_error(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A 429 with ``Retry-After: 12`` reports 12 seconds and the proxy reason."""
    fixture_server.route(QUOTE_SHORT_PATH, b"slow down", status=429, content_type="text/plain", headers=VALET_HEADERS)
    error = raise_status(client, errors)
    assert error.status == 429
    assert error.retry_after == 12.0
    assert error.proxy_error == "rate_limited"
    assert error.headers == ALLOWLISTED
    assert str(error) == "provider returned HTTP status 429 (endpoint: quote-short): slow down"
    rendered = f"{error!s} {error!r} {error.headers}"
    for secret in ("cookie-secret", "header-secret", "query-secret", "unrelated-value"):
        assert secret not in rendered


def test_provider_message_keeps_allowlisted_headers(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A 200 provider message keeps the same allowlisted headers."""
    fixture_server.route(QUOTE_SHORT_PATH, b"Invalid name", headers=VALET_HEADERS)
    error = raise_status(client, errors)
    assert error.status == 200
    assert error.headers == ALLOWLISTED
    assert error.retry_after == 12.0


def test_http_date_retry_after_is_relative_to_now(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """An HTTP-date ``Retry-After`` becomes the seconds left until that date."""

    retry_at = formatdate(time.time() + 120, usegmt=True)
    fixture_server.route(QUOTE_SHORT_PATH, b"", status=503, headers=(("Retry-After", retry_at),))
    error = raise_status(client, errors)
    assert error.retry_after is not None
    assert 100.0 <= error.retry_after <= 121.0
    assert error.proxy_error is None


def test_status_error_without_allowlisted_headers_reports_none(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """Only non-allowlisted headers leave every header attribute ``None``."""
    fixture_server.route(QUOTE_SHORT_PATH, b"", status=500, headers=(("Set-Cookie", "a=b"),))
    error = raise_status(client, errors)
    assert (error.retry_after, error.headers, error.proxy_error) == (None, None, None)
