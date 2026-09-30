"""Runtime contract of ``client.indexes`` for the constituent methods and the negatives.

The six constituent methods take no arguments and map to the exact paths the
Rust ``indexes_constituent_endpoints.rs`` test pins (``dowjones`` has no
separator). The negative cases cover every argument kind the domain has
(``ticker`` and ``date``) plus the structured status and decode failures.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.indexes import HistoricalIndexConstituent, IndexConstituent


@pytest.mark.parametrize(
    ("method", "path", "fixture", "symbol", "founded", "date_first_added"),
    [
        pytest.param(
            "sp500_constituents",
            "/sp500-constituent",
            "indexes_sp500_constituents.json",
            "HONA",
            "1902/1985",
            datetime.date(2026, 6, 29),
            id="sp500",
        ),
        pytest.param(
            "nasdaq_constituents",
            "/nasdaq-constituent",
            "indexes_nasdaq_constituents.json",
            "ADBE",
            "1982-12-01",
            None,
            id="nasdaq",
        ),
        pytest.param(
            "dow_jones_constituents",
            "/dowjones-constituent",
            "indexes_dow_jones_constituents.json",
            "GOOGL",
            "1998-09-04",
            datetime.date(2026, 6, 29),
            id="dow-jones",
        ),
    ],
)
def test_current_constituents_take_no_arguments(
    client: Any,
    fixture_server: FixtureServer,
    method: str,
    path: str,
    fixture: str,
    symbol: str,
    founded: str,
    date_first_added: datetime.date | None,
) -> None:
    """Each current-constituent method hits its own path with no query and decodes the date fields."""
    fixture_server.route(path, load_fixture(fixture))
    rows = getattr(client.indexes, method)()

    assert fixture_server.requests[0].target == path
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, IndexConstituent)
    assert row.symbol == symbol
    assert row.founded == founded
    assert row.date_first_added == date_first_added
    assert row.cik.startswith("000")
    assert row.head_quarter


@pytest.mark.parametrize("founded", ["1994", "1902/1985", "1902/1985/2001"])
def test_constituent_founded_keeps_the_year_forms_verbatim(
    client: Any, fixture_server: FixtureServer, founded: str
) -> None:
    """``founded`` is the provider's founding year or years, returned as the exact string."""
    body = load_fixture("indexes_sp500_constituents.json")
    body[0]["founded"] = founded
    fixture_server.route("/sp500-constituent", body)

    assert client.indexes.sp500_constituents()[0].founded == founded


@pytest.mark.parametrize(
    ("method", "path", "fixture", "symbol", "removed_ticker", "date"),
    [
        pytest.param(
            "historical_sp500_constituents",
            "/historical-sp500-constituent",
            "indexes_historical_sp500_constituents.json",
            "HONA",
            "CAG",
            datetime.date(2026, 6, 29),
            id="sp500",
        ),
        pytest.param(
            "historical_nasdaq_constituents",
            "/historical-nasdaq-constituent",
            "indexes_historical_nasdaq_constituents.json",
            "SPCX",
            None,
            datetime.date(2026, 7, 6),
            id="nasdaq",
        ),
        pytest.param(
            "historical_dow_jones_constituents",
            "/historical-dowjones-constituent",
            "indexes_historical_dow_jones_constituents.json",
            "GOOGL",
            "VZ",
            datetime.date(2026, 6, 29),
            id="dow-jones",
        ),
    ],
)
def test_historical_constituents_take_no_arguments(
    client: Any,
    fixture_server: FixtureServer,
    method: str,
    path: str,
    fixture: str,
    symbol: str,
    removed_ticker: str | None,
    date: datetime.date,
) -> None:
    """Each historical method decodes ``date`` as a date and keeps ``date_added`` as opaque text."""
    fixture_server.route(path, load_fixture(fixture))
    rows = getattr(client.indexes, method)()

    assert fixture_server.requests[0].target == path
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, HistoricalIndexConstituent)
    assert row.symbol == symbol
    assert row.removed_ticker == removed_ticker
    assert row.date == date
    assert isinstance(row.date_added, str)
    assert (row.reason is None) == (symbol == "GOOGL")


def test_historical_constituents_decode_null_members_and_blank_removed_ticker_as_none(
    client: Any, fixture_server: FixtureServer
) -> None:
    """Null ``addedSecurity`` and ``reason`` and a blank ``removedTicker`` decode as ``None``."""
    row = {
        "addedSecurity": None,
        "date": "2026-07-06",
        "dateAdded": "July 7, 2026",
        "reason": None,
        "removedSecurity": "",
        "removedTicker": "",
        "symbol": "SPCX",
    }
    fixture_server.route("/historical-nasdaq-constituent", [row])
    decoded = client.indexes.historical_nasdaq_constituents()[0]

    assert decoded.added_security is None
    assert decoded.reason is None
    assert decoded.removed_ticker is None
    assert decoded.removed_security == ""


@pytest.mark.parametrize(
    "method",
    [
        "list",
        "sp500_constituents",
        "nasdaq_constituents",
        "dow_jones_constituents",
        "historical_sp500_constituents",
        "historical_nasdaq_constituents",
        "historical_dow_jones_constituents",
    ],
)
def test_query_less_methods_reject_arguments(client: Any, fixture_server: FixtureServer, method: str) -> None:
    """A query-less method called with a symbol is a ``TypeError`` before any request."""
    with pytest.raises(TypeError):
        getattr(client.indexes, method)("^VIX")
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["quote", "quote_short", "chart_light", "chart_one_minute"])
def test_blank_symbol_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str
) -> None:
    """A whitespace-only ``symbol`` is rejected locally with the argument name as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.indexes, method)("  ")
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
        client.indexes.chart_full("^VIX", **{keyword: value})
    error = raised.value
    assert str(error) == f"{keyword}: value must be a valid YYYY-MM-DD date"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_status_error_names_the_endpoint(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-success status carries the libfmp endpoint id, status, and body."""
    fixture_server.route("/sp500-constituent", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.indexes.sp500_constituents()
    error = raised.value
    assert error.endpoint == "sp500-constituent"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_decode_error_names_the_chart_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-array body on a chart route surfaces as a decode error with the endpoint id."""
    fixture_server.route("/historical-price-eod/light", {})
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.indexes.chart_light("^VIX")
    assert raised.value.endpoint == "historical-price-eod/light"
