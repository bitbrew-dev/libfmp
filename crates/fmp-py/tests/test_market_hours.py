"""Runtime contract of ``client.market_hours`` for the three exchange-hours methods.

One test per argument shape routes the documented fixture body, calls the
method, and asserts the exact request target plus a few typed fields
(including the ``datetime.date`` holiday field). The expected targets are the
ones the Rust ``market_hours_endpoints.rs`` test pins, including the
form-encoded exchange code and the leading zeros of the opaque timestamp. The
negatives cover every argument kind the domain has (``exchange_code``,
``market_hours_timestamp``, and ``date``) plus the structured status and
decode failures.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.market_hours import ExchangeHoliday, ExchangeMarketHours, MarketHoursNamespace

TIMESTAMP = "001769527402"
FROM = datetime.date(2025, 4, 27)
TO = datetime.date(2026, 4, 27)
DATE_QUERY = "from=2025-04-27&to=2026-04-27"
SPACED_EXCHANGE = "NASDAQ / Global"
SPACED_EXCHANGE_ENCODED = "NASDAQ+%2F+Global"


def test_market_hours_namespace_is_the_generated_type(client: Any) -> None:
    """``client.market_hours`` is the generated flat namespace class."""
    assert isinstance(client.market_hours, MarketHoursNamespace)


def test_exchange_market_hours_with_only_the_exchange(client: Any, fixture_server: FixtureServer) -> None:
    """``exchange_market_hours`` sends only ``exchange`` and decodes the raw hour strings."""
    fixture_server.route("/exchange-market-hours", load_fixture("exchange_market_hours.json"))
    rows = client.market_hours.exchange_market_hours("NASDAQ")

    assert fixture_server.requests[0].target == "/exchange-market-hours?exchange=NASDAQ"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, ExchangeMarketHours)
    assert row.exchange == "NASDAQ"
    assert row.name == "NASDAQ"
    assert row.opening_hour == "09:30 AM -04:00"
    assert row.closing_hour == "04:00 PM -04:00"
    assert row.timezone == "America/New_York"
    assert row.is_market_open is True


def test_exchange_market_hours_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``exchange_market_hours`` form-encodes the exchange and keeps the timestamp's leading zeros."""
    fixture_server.route("/exchange-market-hours", load_fixture("exchange_market_hours.json"))
    rows = client.market_hours.exchange_market_hours(SPACED_EXCHANGE, timestamp=TIMESTAMP)

    assert (
        fixture_server.requests[0].target
        == f"/exchange-market-hours?exchange={SPACED_EXCHANGE_ENCODED}&timestamp={TIMESTAMP}"
    )
    assert fixture_server.requests[0].query == {"exchange": [SPACED_EXCHANGE], "timestamp": [TIMESTAMP]}
    assert len(rows) == 1
    assert isinstance(rows[0].is_market_open, bool)


def test_holidays_by_exchange_with_only_the_exchange(client: Any, fixture_server: FixtureServer) -> None:
    """``holidays_by_exchange`` sends only ``exchange`` and decodes the date plus the null adjusted times."""
    fixture_server.route("/holidays-by-exchange", load_fixture("holidays_by_exchange.json"))
    rows = client.market_hours.holidays_by_exchange("NASDAQ")

    assert fixture_server.requests[0].target == "/holidays-by-exchange?exchange=NASDAQ"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, ExchangeHoliday)
    assert row.exchange == "NASDAQ"
    assert row.date == datetime.date(2026, 7, 3)
    assert row.name == "Independence Day"
    assert row.is_closed is True
    assert row.adj_open_time is None
    assert row.adj_close_time is None


def test_holidays_by_exchange_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``holidays_by_exchange`` encodes ``exchange``, ``from``, ``to`` in that order and accepts dates."""
    fixture_server.route("/holidays-by-exchange", load_fixture("holidays_by_exchange.json"))
    rows = client.market_hours.holidays_by_exchange(SPACED_EXCHANGE, from_=FROM, to=TO)

    assert fixture_server.requests[0].target == f"/holidays-by-exchange?exchange={SPACED_EXCHANGE_ENCODED}&{DATE_QUERY}"
    assert len(rows) == 1
    assert isinstance(rows[0].date, datetime.date)


@pytest.mark.parametrize(
    ("keywords", "target"),
    [
        ({"from_": "2025-04-27"}, "/holidays-by-exchange?exchange=NASDAQ&from=2025-04-27"),
        ({"to": "2026-04-27"}, "/holidays-by-exchange?exchange=NASDAQ&to=2026-04-27"),
    ],
)
def test_holidays_by_exchange_dates_are_independent(
    client: Any, fixture_server: FixtureServer, keywords: dict[str, str], target: str
) -> None:
    """Each holiday date bound is sent alone after ``exchange`` when the other one is left out."""
    fixture_server.route("/holidays-by-exchange", load_fixture("holidays_by_exchange.json"))
    rows = client.market_hours.holidays_by_exchange("NASDAQ", **keywords)

    assert fixture_server.requests[0].target == target
    assert len(rows) == 1


def test_all_exchange_market_hours_with_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``all_exchange_market_hours`` maps to ``/all-exchange-market-hours`` with no query string."""
    fixture_server.route("/all-exchange-market-hours", load_fixture("all_exchange_market_hours.json"))
    rows = client.market_hours.all_exchange_market_hours()

    assert fixture_server.requests[0].target == "/all-exchange-market-hours"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, ExchangeMarketHours)
    assert row.exchange == "ASX"
    assert row.name == "Australian Securities Exchange"
    assert row.opening_hour == "10:00 AM +10:00"
    assert row.timezone == "Australia/Sydney"
    assert row.is_market_open is False


def test_all_exchange_market_hours_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``all_exchange_market_hours`` sends the opaque timestamp unchanged."""
    fixture_server.route("/all-exchange-market-hours", load_fixture("all_exchange_market_hours.json"))
    rows = client.market_hours.all_exchange_market_hours(timestamp=TIMESTAMP)

    assert fixture_server.requests[0].target == f"/all-exchange-market-hours?timestamp={TIMESTAMP}"
    assert len(rows) == 1


def test_from_keyword_is_renamed_but_the_wire_key_is_not(client: Any, fixture_server: FixtureServer) -> None:
    """The Python keyword is ``from_``; ``from`` is rejected and the wire key stays ``from``."""
    fixture_server.route("/holidays-by-exchange", load_fixture("holidays_by_exchange.json"))
    with pytest.raises(TypeError, match="unexpected keyword argument 'from'"):
        client.market_hours.holidays_by_exchange("NASDAQ", **{"from": "2025-04-27"})
    assert fixture_server.requests == []

    client.market_hours.holidays_by_exchange("NASDAQ", from_="2025-04-27")
    assert fixture_server.requests[0].query == {"exchange": ["NASDAQ"], "from": ["2025-04-27"]}
    assert "from_" not in fixture_server.requests[0].raw_query


def test_optionals_are_keyword_only(client: Any, fixture_server: FixtureServer) -> None:
    """A positional timestamp or date is a ``TypeError``, never a silent ``timestamp`` or ``from``."""
    with pytest.raises(TypeError):
        client.market_hours.exchange_market_hours("NASDAQ", TIMESTAMP)
    with pytest.raises(TypeError):
        client.market_hours.holidays_by_exchange("NASDAQ", "2025-04-27")
    with pytest.raises(TypeError):
        client.market_hours.all_exchange_market_hours(TIMESTAMP)
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["exchange_market_hours", "holidays_by_exchange"])
def test_exchange_is_required(client: Any, fixture_server: FixtureServer, method: str) -> None:
    """The two per-exchange methods without an exchange are a ``TypeError`` before any request."""
    with pytest.raises(TypeError):
        getattr(client.market_hours, method)()
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["exchange_market_hours", "holidays_by_exchange"])
@pytest.mark.parametrize(
    ("value", "message"),
    [
        ("", "exchange: value must not be empty or whitespace-only"),
        ("  ", "exchange: value must not be empty or whitespace-only"),
        ("NAS\nDAQ", "exchange: value must not contain control characters"),
    ],
)
def test_invalid_exchange_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, value: str, message: str
) -> None:
    """A blank or control-character ``exchange`` is rejected locally with the argument name as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.market_hours, method)(value)
    error = raised.value
    assert str(error) == message
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("method", "positional"),
    [("exchange_market_hours", ("NASDAQ",)), ("all_exchange_market_hours", ())],
)
@pytest.mark.parametrize(
    ("value", "message"),
    [
        ("", "timestamp: value must not be empty or whitespace-only"),
        ("\t", "timestamp: value must not be empty or whitespace-only"),
        ("0017\n69527402", "timestamp: value must not contain control characters"),
    ],
)
def test_invalid_timestamp_names_the_argument(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    positional: tuple[str, ...],
    value: str,
    message: str,
) -> None:
    """A blank or control-character ``timestamp`` is rejected locally on both methods, prefixed with the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.market_hours, method)(*positional, timestamp=value)
    error = raised.value
    assert str(error) == message
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("keyword", "value"),
    [("from_", "06/03/2026"), ("to", "2026-13-01"), ("from_", ""), ("to", "2026-6-6")],
)
def test_non_iso_holiday_date_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, keyword: str, value: str
) -> None:
    """A date string that is not ``YYYY-MM-DD`` fails locally once the exchange has been accepted."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.market_hours.holidays_by_exchange("NASDAQ", **{keyword: value})
    error = raised.value
    assert str(error) == f"{keyword}: value must be a valid YYYY-MM-DD date"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_status_error_names_the_endpoint(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-success status carries the libfmp endpoint id, status, and body."""
    fixture_server.route("/exchange-market-hours", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.market_hours.exchange_market_hours("NASDAQ")
    error = raised.value
    assert error.endpoint == "exchange-market-hours"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_status_error_names_the_all_exchange_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A failing all-exchange route reports the ``all-exchange-market-hours`` endpoint id and the raw body."""
    fixture_server.route("/all-exchange-market-hours", b"not-json", status=500, content_type="text/plain")
    with pytest.raises(errors.FmpStatusError) as raised:
        client.market_hours.all_exchange_market_hours(timestamp=TIMESTAMP)
    assert raised.value.endpoint == "all-exchange-market-hours"
    assert raised.value.status == 500
    assert raised.value.body == "not-json"


def test_decode_error_names_the_holidays_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-array body on the holidays route surfaces as a decode error with the endpoint id."""
    fixture_server.route("/holidays-by-exchange", {})
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.market_hours.holidays_by_exchange("NASDAQ")
    assert raised.value.endpoint == "holidays-by-exchange"


def test_decode_error_names_the_exchange_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A well-formed but wrongly shaped row on the exchange route is a decode error."""
    fixture_server.route("/exchange-market-hours", [{"exchange": "NASDAQ"}])
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.market_hours.exchange_market_hours("NASDAQ")
    assert raised.value.endpoint == "exchange-market-hours"
