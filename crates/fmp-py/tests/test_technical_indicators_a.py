"""Runtime contract of ``client.technical_indicators``: one test per method.

All nine methods share one query shape (``symbol``, ``period_length``,
``timeframe``, optional ``from_`` and ``to``). Each test routes the documented
fixture body, calls the method with one argument shape, and asserts the exact
request target plus the ``datetime.datetime`` bar timestamp, the integer
volume, and the indicator value. The expected targets are the ones the Rust
``technical_indicator_*_endpoints.rs`` tests pin.
"""

import datetime
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.technical_indicators import (
    AverageDirectionalIndexBar,
    DoubleExponentialMovingAverageBar,
    ExponentialMovingAverageBar,
    RelativeStrengthIndexBar,
    SimpleMovingAverageBar,
    StandardDeviationBar,
    TechnicalIndicatorsNamespace,
    TripleExponentialMovingAverageBar,
    WeightedMovingAverageBar,
    WilliamsBar,
)

BAR_TIMESTAMP = datetime.datetime(2026, 7, 30, 0, 0, 0)
BAR_VOLUME = 29_207_295
FROM_MARCH = datetime.date(2026, 3, 1)
TO_JUNE = datetime.date(2026, 6, 1)


def assert_common_bar_fields(row: Any) -> None:
    """Every indicator row carries the same OHLCV fields from the shared fixture shape."""
    assert row.date == BAR_TIMESTAMP
    assert row.open == pytest.approx(333.13)
    assert row.high == pytest.approx(334.48)
    assert row.low == pytest.approx(329.59)
    assert row.close == pytest.approx(332.39)
    assert row.volume == BAR_VOLUME


def test_technical_indicators_namespace_is_the_generated_type(client: Any) -> None:
    """``client.technical_indicators`` is the generated namespace class."""
    assert isinstance(client.technical_indicators, TechnicalIndicatorsNamespace)


def test_simple_moving_average_with_both_dates_and_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``simple_moving_average`` form-encodes the ticker and appends ``from`` then ``to``."""
    fixture_server.route("/technical-indicators/sma", load_fixture("technical_indicator_sma.json"))
    rows = client.technical_indicators.simple_moving_average(
        "BRK.B / Class A", 10, "1day", from_=FROM_MARCH, to=TO_JUNE
    )

    assert fixture_server.requests[0].target == (
        "/technical-indicators/sma?symbol=BRK.B+%2F+Class+A&periodLength=10&timeframe=1day"
        "&from=2026-03-01&to=2026-06-01"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SimpleMovingAverageBar)
    assert_common_bar_fields(row)
    assert row.sma == pytest.approx(331.621)


def test_exponential_moving_average_with_from_only(client: Any, fixture_server: FixtureServer) -> None:
    """``exponential_moving_average`` sends ``from`` alone on the hourly timeframe."""
    fixture_server.route("/technical-indicators/ema", load_fixture("technical_indicator_ema.json"))
    rows = client.technical_indicators.exponential_moving_average("AAPL", 10, "1hour", from_="2026-03-01")

    assert fixture_server.requests[0].target == (
        "/technical-indicators/ema?symbol=AAPL&periodLength=10&timeframe=1hour&from=2026-03-01"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, ExponentialMovingAverageBar)
    assert_common_bar_fields(row)
    assert row.ema == pytest.approx(331.1209325826155)


def test_weighted_moving_average_with_to_only(client: Any, fixture_server: FixtureServer) -> None:
    """``weighted_moving_average`` sends ``to`` alone on the four-hour timeframe."""
    fixture_server.route("/technical-indicators/wma", load_fixture("technical_indicator_wma.json"))
    rows = client.technical_indicators.weighted_moving_average("AAPL", 10, "4hour", to=TO_JUNE)

    assert fixture_server.requests[0].target == (
        "/technical-indicators/wma?symbol=AAPL&periodLength=10&timeframe=4hour&to=2026-06-01"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, WeightedMovingAverageBar)
    assert_common_bar_fields(row)
    assert row.wma == pytest.approx(333.21345454545457)


def test_double_exponential_moving_average_with_from_only(client: Any, fixture_server: FixtureServer) -> None:
    """``double_exponential_moving_average`` keeps the spaced ticker and a lone ``from``."""
    fixture_server.route("/technical-indicators/dema", load_fixture("technical_indicator_dema.json"))
    rows = client.technical_indicators.double_exponential_moving_average("BRK.B / Class A", 10, "1day", from_=TO_JUNE)

    assert fixture_server.requests[0].target == (
        "/technical-indicators/dema?symbol=BRK.B+%2F+Class+A&periodLength=10&timeframe=1day&from=2026-06-01"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, DoubleExponentialMovingAverageBar)
    assert_common_bar_fields(row)
    assert row.dema == pytest.approx(337.7659642917977)


def test_triple_exponential_moving_average_with_to_only(client: Any, fixture_server: FixtureServer) -> None:
    """``triple_exponential_moving_average`` sends ``to`` alone as an ISO string."""
    fixture_server.route("/technical-indicators/tema", load_fixture("technical_indicator_tema.json"))
    rows = client.technical_indicators.triple_exponential_moving_average("AAPL", 10, "4hour", to="2026-03-01")

    assert fixture_server.requests[0].target == (
        "/technical-indicators/tema?symbol=AAPL&periodLength=10&timeframe=4hour&to=2026-03-01"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, TripleExponentialMovingAverageBar)
    assert_common_bar_fields(row)
    assert row.tema == pytest.approx(337.09671042323214)


def test_relative_strength_index_on_one_minute_bars(client: Any, fixture_server: FixtureServer) -> None:
    """``relative_strength_index`` accepts the ``1min`` timeframe with a lone ``from``."""
    fixture_server.route("/technical-indicators/rsi", load_fixture("technical_indicator_rsi.json"))
    rows = client.technical_indicators.relative_strength_index("BRK.B / Class A", 10, "1min", from_=TO_JUNE)

    assert fixture_server.requests[0].target == (
        "/technical-indicators/rsi?symbol=BRK.B+%2F+Class+A&periodLength=10&timeframe=1min&from=2026-06-01"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, RelativeStrengthIndexBar)
    assert_common_bar_fields(row)
    assert row.rsi == pytest.approx(59.55175118203601)


def test_standard_deviation_uses_the_unhyphenated_path(client: Any, fixture_server: FixtureServer) -> None:
    """``standard_deviation`` maps to ``/technical-indicators/standarddeviation`` and snake-cases the metric."""
    fixture_server.route(
        "/technical-indicators/standarddeviation", load_fixture("technical_indicator_standard_deviation.json")
    )
    rows = client.technical_indicators.standard_deviation("AAPL", 10, "5min", to=FROM_MARCH)

    assert fixture_server.requests[0].target == (
        "/technical-indicators/standarddeviation?symbol=AAPL&periodLength=10&timeframe=5min&to=2026-03-01"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StandardDeviationBar)
    assert_common_bar_fields(row)
    assert row.standard_deviation == pytest.approx(5.675893674127448)


def test_williams_with_reversed_dates_and_negative_value(client: Any, fixture_server: FixtureServer) -> None:
    """``williams`` forwards ``from`` after ``to`` unchecked and decodes a negative oscillator."""
    fixture_server.route("/technical-indicators/williams", load_fixture("technical_indicator_williams.json"))
    rows = client.technical_indicators.williams("AAPL", 10, "15min", from_=TO_JUNE, to=FROM_MARCH)

    assert fixture_server.requests[0].target == (
        "/technical-indicators/williams?symbol=AAPL&periodLength=10&timeframe=15min&from=2026-06-01&to=2026-03-01"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, WilliamsBar)
    assert_common_bar_fields(row)
    assert row.williams == pytest.approx(-48.29500396510714)


def test_average_directional_index_without_dates(client: Any, fixture_server: FixtureServer) -> None:
    """``average_directional_index`` sends only the three required parameters."""
    fixture_server.route("/technical-indicators/adx", load_fixture("technical_indicator_adx.json"))
    rows = client.technical_indicators.average_directional_index("AAPL", 10, "30min")

    assert fixture_server.requests[0].target == "/technical-indicators/adx?symbol=AAPL&periodLength=10&timeframe=30min"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, AverageDirectionalIndexBar)
    assert_common_bar_fields(row)
    assert row.adx == pytest.approx(34.69458756515438)
