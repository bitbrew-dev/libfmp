"""Runtime contract of ``client.forex`` for the negatives and the error mapping.

The negative cases cover every argument kind the domain has (``ticker`` and
``date``) plus the query-less ``list`` rejecting arguments and the structured
status and decode failures. The endpoint ids in the errors are the ones the
Rust ``asset_catalog_quote_endpoints.rs`` and ``asset_history_endpoints.rs``
tests pin.
"""

from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer


def test_list_rejects_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """The query-less ``list`` called with a symbol is a ``TypeError`` before any request."""
    with pytest.raises(TypeError):
        client.forex.list("EURUSD")
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["quote", "quote_short", "chart_light", "chart_one_minute"])
def test_blank_symbol_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str
) -> None:
    """A whitespace-only ``symbol`` is rejected locally with the argument name as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.forex, method)("  ")
    error = raised.value
    assert str(error) == "symbol: value must not be empty or whitespace-only"
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("keyword", "value"),
    [("from_", "06/03/2026"), ("to", "2026-13-01"), ("from_", ""), ("to", "2026-6-6")],
)
def test_non_iso_date_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, keyword: str, value: str
) -> None:
    """A date string that is not ``YYYY-MM-DD`` fails locally, prefixed with the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.forex.chart_full("EURUSD", **{keyword: value})
    error = raised.value
    assert str(error) == f"{keyword}: value must be a valid YYYY-MM-DD date"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_status_error_names_the_endpoint(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-success status carries the libfmp endpoint id, status, and body."""
    fixture_server.route("/forex-list", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.forex.list()
    error = raised.value
    assert error.endpoint == "forex-list"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_status_error_names_the_shared_quote_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A failing shared ``/quote`` route reports the generic ``quote`` endpoint id."""
    fixture_server.route("/quote", b"not-json", status=500, content_type="text/plain")
    with pytest.raises(errors.FmpStatusError) as raised:
        client.forex.quote("EURUSD")
    assert raised.value.endpoint == "quote"
    assert raised.value.status == 500
    assert raised.value.body == "not-json"


def test_decode_error_names_the_chart_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-array body on a chart route surfaces as a decode error with the endpoint id."""
    fixture_server.route("/historical-price-eod/light", {})
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.forex.chart_light("EURUSD")
    assert raised.value.endpoint == "historical-price-eod/light"


def test_decode_error_names_the_intraday_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A well-formed but wrongly shaped bar on an intraday route is a decode error."""
    fixture_server.route("/historical-chart/1min", [{"date": "2026-07-30 13:17:00"}])
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.forex.chart_one_minute("EURUSD")
    assert raised.value.endpoint == "historical-chart/1min"
