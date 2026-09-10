"""Argument shapes, local validation, and error mapping for ``client.technical_indicators``.

The nine methods share one query, so the shapes are exercised on a couple of
representative methods: every documented timeframe spelling, the period-length
bounds, the two date keyword forms, and one negative per argument family whose
message starts with the argument name. Two failure routes prove the status and
decode errors stay structured.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture

TIMEFRAMES = ["1min", "5min", "15min", "30min", "1hour", "4hour", "1day"]


@pytest.mark.parametrize("timeframe", TIMEFRAMES)
def test_every_timeframe_spelling_reaches_the_wire(client: Any, fixture_server: FixtureServer, timeframe: str) -> None:
    """All seven documented timeframes are forwarded verbatim without dates."""
    fixture_server.route("/technical-indicators/sma", load_fixture("technical_indicator_sma.json"))
    rows = client.technical_indicators.simple_moving_average("AAPL", 10, timeframe)

    assert fixture_server.requests[0].target == (
        f"/technical-indicators/sma?symbol=AAPL&periodLength=10&timeframe={timeframe}"
    )
    assert len(rows) == 1


def test_timeframe_is_matched_case_insensitively(client: Any, fixture_server: FixtureServer) -> None:
    """An upper-case timeframe is normalised to the documented lower-case wire spelling."""
    fixture_server.route("/technical-indicators/rsi", load_fixture("technical_indicator_rsi.json"))
    client.technical_indicators.relative_strength_index("AAPL", 14, "1DAY")

    assert fixture_server.requests[0].target == "/technical-indicators/rsi?symbol=AAPL&periodLength=14&timeframe=1day"


def test_period_length_accepts_the_full_u32_range(client: Any, fixture_server: FixtureServer) -> None:
    """``period_length`` may be any strictly positive 32-bit value and is sent as digits."""
    fixture_server.route("/technical-indicators/adx", load_fixture("technical_indicator_adx.json"))
    client.technical_indicators.average_directional_index("AAPL", 4_294_967_295, "1day")

    assert fixture_server.requests[0].target == (
        "/technical-indicators/adx?symbol=AAPL&periodLength=4294967295&timeframe=1day"
    )


def test_dates_accept_date_objects_and_iso_strings_alike(client: Any, fixture_server: FixtureServer) -> None:
    """``from_`` as a ``datetime.date`` and ``to`` as a string encode identically."""
    fixture_server.route("/technical-indicators/wma", load_fixture("technical_indicator_wma.json"))
    client.technical_indicators.weighted_moving_average(
        "AAPL", 20, "1day", from_=datetime.date(2026, 1, 2), to="2026-02-03"
    )

    assert fixture_server.requests[0].target == (
        "/technical-indicators/wma?symbol=AAPL&periodLength=20&timeframe=1day&from=2026-01-02&to=2026-02-03"
    )


def test_optional_dates_are_keyword_only(client: Any) -> None:
    """A fourth positional argument is rejected by the signature, not sent to the server."""
    with pytest.raises(TypeError):
        client.technical_indicators.simple_moving_average("AAPL", 10, "1day", "2026-01-02")


@pytest.mark.parametrize(
    "method",
    [
        "simple_moving_average",
        "exponential_moving_average",
        "weighted_moving_average",
        "double_exponential_moving_average",
        "triple_exponential_moving_average",
        "relative_strength_index",
        "standard_deviation",
        "williams",
        "average_directional_index",
    ],
)
def test_blank_symbol_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str
) -> None:
    """A whitespace-only ``symbol`` is rejected locally on every method with the argument name as prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.technical_indicators, method)("  ", 10, "1day")
    assert str(raised.value) == "symbol: value must not be empty or whitespace-only"
    assert fixture_server.requests == []


@pytest.mark.parametrize("period_length", [0, -1, 4_294_967_296])
def test_period_length_outside_the_positive_u32_range_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, period_length: int
) -> None:
    """Zero, negative, and over-range period lengths fail locally, prefixed with ``period_length``."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.technical_indicators.exponential_moving_average("AAPL", period_length, "1day")
    assert str(raised.value) == "period_length: period length must be a positive integer"
    assert fixture_server.requests == []


def test_non_integer_period_length_is_a_type_error(client: Any, fixture_server: FixtureServer) -> None:
    """A string period length is a ``TypeError`` from the signature, not a validation error."""
    with pytest.raises(TypeError):
        client.technical_indicators.exponential_moving_average("AAPL", "10", "1day")
    assert fixture_server.requests == []


@pytest.mark.parametrize("timeframe", ["2min", "1 day", "daily", ""])
def test_unknown_timeframe_names_the_argument_and_lists_the_choices(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, timeframe: str
) -> None:
    """An undocumented timeframe fails locally with the accepted spellings in the message."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.technical_indicators.williams("AAPL", 10, timeframe)
    assert str(raised.value) == "timeframe: timeframe must be one of 1min, 5min, 15min, 30min, 1hour, 4hour, 1day"
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
        client.technical_indicators.standard_deviation("AAPL", 10, "1day", **{keyword: value})
    assert str(raised.value) == f"{keyword}: value must be a valid YYYY-MM-DD date"
    assert fixture_server.requests == []


def test_status_failure_is_structured(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A ``401`` on the indicator path raises ``FmpStatusError`` carrying status and endpoint."""
    fixture_server.route("/technical-indicators/sma", {"Error Message": "Invalid API KEY."}, status=401)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.technical_indicators.simple_moving_average("AAPL", 10, "1day")
    error = raised.value
    assert error.status == 401
    assert error.endpoint == "technical-indicators/sma"
    assert len(fixture_server.requests) == 1


def test_decode_failure_is_structured(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-JSON ``200`` body raises ``FmpDecodeError`` that still names the endpoint."""
    fixture_server.route("/technical-indicators/adx", b"not-json", content_type="text/plain")
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.technical_indicators.average_directional_index("AAPL", 10, "1day")
    assert raised.value.endpoint == "technical-indicators/adx"


def test_empty_array_decodes_to_no_rows(client: Any, fixture_server: FixtureServer) -> None:
    """An empty provider array is an empty Python list, not an error."""
    fixture_server.route("/technical-indicators/tema", [])
    rows = client.technical_indicators.triple_exponential_moving_average("AAPL", 10, "1day")
    assert rows == []
