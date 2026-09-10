"""Runtime contract of ``client.chart`` for the six intraday methods and the negatives.

The expected targets are the ones the Rust ``chart_intraday_endpoints.rs``
test pins: ``historical-chart/1min``, ``5min``, ``15min``, ``30min``,
``1hour``, ``4hour`` with ``symbol``, ``from``, ``to``, ``nonadjusted``,
``extended`` in that order, booleans as lowercase strings, explicit ``False``
preserved. Every intraday row carries a ``datetime.datetime``. The negatives
cover one invalid value per argument-kind family the domain uses: ``ticker``,
``date``, and the ``bool``-typed ``boolean`` kind, which is a shape error
(``TypeError``) rather than a validation error; plus the structured status and
decode failures.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.chart import StockChartIntradayBar

INTRADAY_FROM = datetime.date(2024, 1, 1)
INTRADAY_TO = datetime.date(2024, 3, 1)
DATE_QUERY = "symbol=AAPL&from=2024-01-01&to=2024-03-01"


def test_one_minute_without_options_sends_only_the_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``one_minute`` maps to ``/historical-chart/1min`` and decodes a ``datetime`` row."""
    fixture_server.route("/historical-chart/1min", load_fixture("stock_chart_one_minute.json"))
    rows = client.chart.one_minute("AAPL")

    assert fixture_server.requests[0].path == "/historical-chart/1min"
    assert fixture_server.requests[0].target == "/historical-chart/1min?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartIntradayBar)
    assert row.date == datetime.datetime(2026, 7, 30, 13, 16)
    assert row.open == pytest.approx(332.4)
    assert row.low == pytest.approx(332.27499)
    assert row.high == pytest.approx(332.48)
    assert row.close == pytest.approx(332.47)
    assert row.volume == 67660


def test_five_minutes_with_from_and_nonadjusted_false(client: Any, fixture_server: FixtureServer) -> None:
    """``five_minutes`` keeps an explicit ``nonadjusted=False`` and encodes it as ``false``."""
    fixture_server.route("/historical-chart/5min", load_fixture("stock_chart_five_minutes.json"))
    rows = client.chart.five_minutes("AAPL", from_=INTRADAY_FROM, nonadjusted=False)

    assert fixture_server.requests[0].target == "/historical-chart/5min?symbol=AAPL&from=2024-01-01&nonadjusted=false"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartIntradayBar)
    assert row.date == datetime.datetime(2026, 7, 30, 13, 15)
    assert row.volume == 123020
    assert row.close == pytest.approx(332.31989)


def test_fifteen_minutes_with_to_and_extended_false(client: Any, fixture_server: FixtureServer) -> None:
    """``fifteen_minutes`` sends ``to`` and an explicit ``extended=false`` without ``from``."""
    fixture_server.route("/historical-chart/15min", load_fixture("stock_chart_fifteen_minutes.json"))
    rows = client.chart.fifteen_minutes("AAPL", to="2024-03-01", extended=False)

    assert fixture_server.requests[0].target == "/historical-chart/15min?symbol=AAPL&to=2024-03-01&extended=false"
    assert len(rows) == 1
    row = rows[0]
    assert row.date == datetime.datetime(2026, 7, 30, 13, 15)
    assert row.open == pytest.approx(332.655)
    assert row.high == pytest.approx(332.755)


def test_thirty_minutes_with_every_option_false(client: Any, fixture_server: FixtureServer) -> None:
    """``thirty_minutes`` encodes both flags as ``false`` after the two dates."""
    fixture_server.route("/historical-chart/30min", load_fixture("stock_chart_thirty_minutes.json"))
    rows = client.chart.thirty_minutes("AAPL", from_=INTRADAY_FROM, to=INTRADAY_TO, nonadjusted=False, extended=False)

    assert fixture_server.requests[0].target == f"/historical-chart/30min?{DATE_QUERY}&nonadjusted=false&extended=false"
    assert len(rows) == 1
    row = rows[0]
    assert row.date == datetime.datetime(2026, 7, 30, 13, 0)
    assert row.volume == 980442
    assert row.low == pytest.approx(331.71)


def test_one_hour_with_every_option_true(client: Any, fixture_server: FixtureServer) -> None:
    """``one_hour`` encodes both flags as ``true`` after the two dates."""
    fixture_server.route("/historical-chart/1hour", load_fixture("stock_chart_one_hour.json"))
    rows = client.chart.one_hour("AAPL", from_="2024-01-01", to=INTRADAY_TO, nonadjusted=True, extended=True)

    assert fixture_server.requests[0].target == f"/historical-chart/1hour?{DATE_QUERY}&nonadjusted=true&extended=true"
    assert fixture_server.requests[0].query["nonadjusted"] == ["true"]
    assert len(rows) == 1
    row = rows[0]
    assert row.date == datetime.datetime(2026, 7, 30, 12, 30)
    assert row.volume == 3285503
    assert row.open == pytest.approx(332.14)


def test_four_hours_with_mixed_flags(client: Any, fixture_server: FixtureServer) -> None:
    """``four_hours`` encodes ``nonadjusted=false`` then ``extended=true`` in documented order."""
    fixture_server.route("/historical-chart/4hour", load_fixture("stock_chart_four_hours.json"))
    rows = client.chart.four_hours("AAPL", from_=INTRADAY_FROM, to="2024-03-01", nonadjusted=False, extended=True)

    assert fixture_server.requests[0].target == f"/historical-chart/4hour?{DATE_QUERY}&nonadjusted=false&extended=true"
    assert len(rows) == 1
    row = rows[0]
    assert row.date == datetime.datetime(2026, 7, 30, 9, 30)
    assert row.volume == 28439347
    assert row.high == pytest.approx(334.26)
    assert row.low == pytest.approx(329.70499)


def test_four_hours_escapes_the_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``four_hours`` form-encodes spaces and slashes in the symbol the way the Rust proxy test pins."""
    fixture_server.route("/historical-chart/4hour", load_fixture("stock_chart_four_hours.json"))
    client.chart.four_hours("BRK.B / Class A")

    assert fixture_server.requests[0].target == "/historical-chart/4hour?symbol=BRK.B+%2F+Class+A"


@pytest.mark.parametrize(
    ("method", "path"),
    [
        ("one_minute", "/historical-chart/1min"),
        ("five_minutes", "/historical-chart/5min"),
        ("fifteen_minutes", "/historical-chart/15min"),
        ("thirty_minutes", "/historical-chart/30min"),
        ("one_hour", "/historical-chart/1hour"),
        ("four_hours", "/historical-chart/4hour"),
    ],
)
def test_intraday_methods_hit_exact_paths_and_preserve_empty_arrays(
    client: Any, fixture_server: FixtureServer, method: str, path: str
) -> None:
    """Each intraday method requests its exact path and returns ``[]`` for the default empty array."""
    assert getattr(client.chart, method)("AAPL") == []
    assert fixture_server.requests[0].target == f"{path}?symbol=AAPL"


@pytest.mark.parametrize("method", ["light", "full", "one_minute", "four_hours"])
def test_blank_symbol_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str
) -> None:
    """A whitespace-only ``symbol`` is rejected locally with the argument name as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.chart, method)("  ")
    error = raised.value
    assert str(error) == "symbol: value must not be empty or whitespace-only"
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("keyword", "value"),
    [("from_", "04/30/2026"), ("to", "2026-13-01"), ("from_", ""), ("to", "2026-7-30")],
)
def test_non_iso_date_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, keyword: str, value: str
) -> None:
    """A date string that is not ``YYYY-MM-DD`` fails locally on both query shapes, prefixed with the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.chart.full("AAPL", **{keyword: value})
    assert str(raised.value) == f"{keyword}: value must be a valid YYYY-MM-DD date"
    with pytest.raises(errors.FmpValidationError) as raised:
        client.chart.one_minute("AAPL", **{keyword: value})
    assert str(raised.value) == f"{keyword}: value must be a valid YYYY-MM-DD date"
    assert raised.value.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize("value", ["false", 0, [True]], ids=["string", "int", "list"])
def test_bool_typed_keywords_reject_non_bool_shapes(client: Any, fixture_server: FixtureServer, value: Any) -> None:
    """``nonadjusted`` and ``extended`` (``boolean``) accept only ``bool``."""
    with pytest.raises(TypeError):
        client.chart.one_minute("AAPL", nonadjusted=value)
    with pytest.raises(TypeError):
        client.chart.four_hours("AAPL", extended=value)
    assert fixture_server.requests == []


def test_eod_methods_reject_the_intraday_flags(client: Any, fixture_server: FixtureServer) -> None:
    """The EOD query has no ``nonadjusted``/``extended`` keywords, so passing one is a ``TypeError``."""
    with pytest.raises(TypeError):
        client.chart.light("AAPL", nonadjusted=True)
    with pytest.raises(TypeError):
        client.chart.dividend_adjusted("AAPL", extended=True)
    assert fixture_server.requests == []


def test_status_error_names_the_endpoint(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-success status carries the libfmp endpoint id, status, and body."""
    fixture_server.route("/historical-chart/1min", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.chart.one_minute("AAPL")
    error = raised.value
    assert error.endpoint == "historical-chart/1min"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_decode_error_names_the_endpoint(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-array body on an EOD route surfaces as a decode error with the endpoint id."""
    fixture_server.route("/historical-price-eod/light", {"not": "an array"})
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.chart.light("AAPL")
    assert raised.value.endpoint == "historical-price-eod/light"
