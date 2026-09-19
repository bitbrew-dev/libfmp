"""Runtime contract of ``client.economics`` for the four macro-data methods.

One test per argument shape routes the documented fixture body, calls the
method, and asserts the exact request target plus a few typed fields
(including the ``datetime.date`` / ``datetime.datetime`` ones). The expected
targets are the ones the Rust ``economics_rates_indicators_endpoints.rs`` and
``economics_calendar_risk_endpoints.rs`` tests pin. The negatives cover every
argument kind the domain has (``economic_indicator``, ``country_code``, and
``date``) plus the query-less ``market_risk_premium`` rejecting arguments and
the structured status and decode failures.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.economics import (
    EconomicCalendarEvent,
    EconomicIndicatorObservation,
    EconomicsNamespace,
    MarketRiskPremium,
    TreasuryRate,
)

FROM = datetime.date(2026, 1, 27)
TO = datetime.date(2026, 4, 27)
DATE_QUERY = "from=2026-01-27&to=2026-04-27"


def test_economics_namespace_is_the_generated_type(client: Any) -> None:
    """``client.economics`` is the generated flat namespace class."""
    assert isinstance(client.economics, EconomicsNamespace)


def test_treasury_rates_with_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``treasury_rates`` maps to ``/treasury-rates`` with no query string and decodes the curve row."""
    fixture_server.route("/treasury-rates", load_fixture("treasury_rates.json"))
    rows = client.economics.treasury_rates()

    assert fixture_server.requests[0].target == "/treasury-rates"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, TreasuryRate)
    assert row.date == datetime.date(2026, 7, 29)
    assert row.month_1 == pytest.approx(3.73)
    assert row.year_10 == pytest.approx(4.67)
    assert row.year_30 == pytest.approx(5.2)


def test_treasury_rates_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``treasury_rates`` encodes ``from`` then ``to`` and accepts ``datetime.date`` values."""
    fixture_server.route("/treasury-rates", load_fixture("treasury_rates.json"))
    rows = client.economics.treasury_rates(from_=FROM, to=TO)

    assert fixture_server.requests[0].target == f"/treasury-rates?{DATE_QUERY}"
    assert len(rows) == 1
    assert isinstance(rows[0].date, datetime.date)


@pytest.mark.parametrize(
    ("keywords", "target"),
    [
        ({"from_": "2026-01-27"}, "/treasury-rates?from=2026-01-27"),
        ({"to": "2026-04-27"}, "/treasury-rates?to=2026-04-27"),
    ],
)
def test_treasury_rates_dates_are_independent(
    client: Any, fixture_server: FixtureServer, keywords: dict[str, str], target: str
) -> None:
    """Each treasury date bound is sent alone when the other one is left out."""
    fixture_server.route("/treasury-rates", load_fixture("treasury_rates.json"))
    rows = client.economics.treasury_rates(**keywords)

    assert fixture_server.requests[0].target == target
    assert len(rows) == 1


def test_indicators_with_a_documented_name(client: Any, fixture_server: FixtureServer) -> None:
    """``indicators`` sends only ``name`` for a documented indicator and decodes the observation row."""
    fixture_server.route("/economic-indicators", load_fixture("economic_indicators.json"))
    rows = client.economics.indicators("GDP")

    assert fixture_server.requests[0].target == "/economic-indicators?name=GDP"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EconomicIndicatorObservation)
    assert row.name == "GDP"
    assert row.date == datetime.date(2025, 10, 1)
    assert row.value == pytest.approx(31_422.526)


def test_indicators_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``indicators`` encodes ``name``, ``from``, ``to`` in that order."""
    fixture_server.route("/economic-indicators", load_fixture("economic_indicators.json"))
    rows = client.economics.indicators("GDP", from_=datetime.date(2025, 4, 27), to="2026-04-27")

    assert fixture_server.requests[0].target == "/economic-indicators?name=GDP&from=2025-04-27&to=2026-04-27"
    assert len(rows) == 1


@pytest.mark.parametrize(
    ("keywords", "target"),
    [
        ({"from_": "2026-01-27"}, "/economic-indicators?name=GDP&from=2026-01-27"),
        ({"to": "2026-04-27"}, "/economic-indicators?name=GDP&to=2026-04-27"),
    ],
)
def test_indicators_dates_are_independent(
    client: Any, fixture_server: FixtureServer, keywords: dict[str, str], target: str
) -> None:
    """Each indicator date bound is sent alone after ``name`` when the other one is left out."""
    fixture_server.route("/economic-indicators", load_fixture("economic_indicators.json"))
    rows = client.economics.indicators("GDP", **keywords)

    assert fixture_server.requests[0].target == target
    assert len(rows) == 1


@pytest.mark.parametrize(
    "name",
    ["realGDP", "CPI", "3MonthOr90DayRatesAndYieldsCertificatesOfDeposit", "futureProviderIndicator"],
)
def test_indicators_preserve_documented_and_open_spellings(
    client: Any, fixture_server: FixtureServer, name: str
) -> None:
    """Documented names and an undocumented (open) name reach the wire unchanged."""
    fixture_server.route("/economic-indicators", load_fixture("economic_indicators.json"))
    rows = client.economics.indicators(name)

    assert fixture_server.requests[0].query == {"name": [name]}
    assert len(rows) == 1


def test_calendar_with_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``calendar`` maps to ``/economic-calendar`` with no query string and decodes a datetime row."""
    fixture_server.route("/economic-calendar", load_fixture("economic_calendar.json"))
    rows = client.economics.calendar()

    assert fixture_server.requests[0].target == "/economic-calendar"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EconomicCalendarEvent)
    assert row.date == datetime.datetime(2026, 7, 29, 3, 30)
    assert row.country == "SG"
    assert row.currency == "SGD"
    assert row.event == "Import Prices YoY (Jun)"
    assert row.actual == pytest.approx(13.6)
    assert row.estimate == pytest.approx(21.0)
    assert row.change == pytest.approx(-4.9)
    assert row.change_percentage == pytest.approx(-26.486)
    assert row.impact == "Low"
    assert row.unit == "%"


def test_calendar_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``calendar`` encodes ``country``, ``from``, ``to`` in that order."""
    fixture_server.route("/economic-calendar", load_fixture("economic_calendar.json"))
    rows = client.economics.calendar(country="US", from_=FROM, to=TO)

    assert fixture_server.requests[0].target == f"/economic-calendar?country=US&{DATE_QUERY}"
    assert len(rows) == 1


@pytest.mark.parametrize(
    ("keywords", "target"),
    [
        ({"country": "US"}, "/economic-calendar?country=US"),
        ({"from_": "2026-01-27"}, "/economic-calendar?from=2026-01-27"),
        ({"to": "2026-04-27"}, "/economic-calendar?to=2026-04-27"),
        ({"country": "US", "from_": "2026-01-27"}, "/economic-calendar?country=US&from=2026-01-27"),
        ({"country": "US", "to": "2026-04-27"}, "/economic-calendar?country=US&to=2026-04-27"),
        ({"from_": "2026-01-27", "to": "2026-04-27"}, f"/economic-calendar?{DATE_QUERY}"),
    ],
)
def test_calendar_filters_are_independent(
    client: Any, fixture_server: FixtureServer, keywords: dict[str, str], target: str
) -> None:
    """Every calendar filter combination omits exactly the filters that were left out."""
    fixture_server.route("/economic-calendar", load_fixture("economic_calendar.json"))
    rows = client.economics.calendar(**keywords)

    assert fixture_server.requests[0].target == target
    assert len(rows) == 1


def test_market_risk_premium_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``market_risk_premium`` maps to ``/market-risk-premium`` with no query string."""
    fixture_server.route("/market-risk-premium", load_fixture("market_risk_premium.json"))
    rows = client.economics.market_risk_premium()

    assert fixture_server.requests[0].target == "/market-risk-premium"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, MarketRiskPremium)
    assert row.country == "Zimbabwe"
    assert row.continent == "Africa"
    assert row.country_risk_premium == pytest.approx(11.66)
    assert row.total_equity_risk_premium == pytest.approx(15.89)


def test_market_risk_premium_rejects_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """The query-less ``market_risk_premium`` called with a country is a ``TypeError`` before any request."""
    with pytest.raises(TypeError):
        client.economics.market_risk_premium("US")
    assert fixture_server.requests == []


def test_from_keyword_is_renamed_but_the_wire_key_is_not(client: Any, fixture_server: FixtureServer) -> None:
    """The Python keyword is ``from_``; ``from`` is rejected and the wire key stays ``from``."""
    fixture_server.route("/treasury-rates", load_fixture("treasury_rates.json"))
    with pytest.raises(TypeError, match="unexpected keyword argument 'from'"):
        client.economics.treasury_rates(**{"from": "2026-01-27"})
    assert fixture_server.requests == []

    client.economics.treasury_rates(from_="2026-01-27")
    assert fixture_server.requests[0].query == {"from": ["2026-01-27"]}
    assert "from_" not in fixture_server.requests[0].raw_query


def test_optionals_are_keyword_only(client: Any, fixture_server: FixtureServer) -> None:
    """A positional date or country is a ``TypeError``, never a silent ``from`` or ``country``."""
    with pytest.raises(TypeError):
        client.economics.treasury_rates("2026-01-27")
    with pytest.raises(TypeError):
        client.economics.indicators("GDP", "2026-01-27")
    with pytest.raises(TypeError):
        client.economics.calendar("US")
    assert fixture_server.requests == []


def test_indicators_requires_the_name(client: Any, fixture_server: FixtureServer) -> None:
    """``indicators`` without a name is a ``TypeError`` before any request."""
    with pytest.raises(TypeError):
        client.economics.indicators()
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("value", "message"),
    [
        ("", "name: value must not be empty or whitespace-only"),
        ("  ", "name: value must not be empty or whitespace-only"),
        ("bad\nname", "name: value must not contain control characters"),
    ],
)
def test_invalid_indicator_name_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, value: str, message: str
) -> None:
    """A blank or control-character ``name`` is rejected locally with the argument name as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.economics.indicators(value)
    error = raised.value
    assert str(error) == message
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize("value", ["", "\t", "U\nS"])
def test_invalid_country_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, value: str
) -> None:
    """A blank or control-character ``country`` is rejected locally, prefixed with the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.economics.calendar(country=value)
    error = raised.value
    assert str(error).startswith("country: value must not ")
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["treasury_rates", "calendar"])
@pytest.mark.parametrize(
    ("keyword", "value"),
    [("from_", "06/03/2026"), ("to", "2026-13-01"), ("from_", ""), ("to", "2026-6-6")],
)
def test_non_iso_date_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, keyword: str, value: str
) -> None:
    """A date string that is not ``YYYY-MM-DD`` fails locally, prefixed with the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.economics, method)(**{keyword: value})
    error = raised.value
    assert str(error) == f"{keyword}: value must be a valid YYYY-MM-DD date"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_non_iso_indicator_date_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """``indicators`` validates its dates the same way once the name has been accepted."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.economics.indicators("GDP", to="2026-13-01")
    assert str(raised.value) == "to: value must be a valid YYYY-MM-DD date"
    assert fixture_server.requests == []


def test_status_error_names_the_endpoint(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-success status carries the libfmp endpoint id, status, and body."""
    fixture_server.route("/treasury-rates", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.economics.treasury_rates()
    error = raised.value
    assert error.endpoint == "treasury-rates"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_status_error_names_the_calendar_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A failing calendar route reports the ``economic-calendar`` endpoint id and the raw body."""
    fixture_server.route("/economic-calendar", b"not-json", status=500, content_type="text/plain")
    with pytest.raises(errors.FmpStatusError) as raised:
        client.economics.calendar(country="US")
    assert raised.value.endpoint == "economic-calendar"
    assert raised.value.status == 500
    assert raised.value.body == "not-json"


def test_decode_error_names_the_indicators_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-array body on the indicators route surfaces as a decode error with the endpoint id."""
    fixture_server.route("/economic-indicators", {})
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.economics.indicators("GDP")
    assert raised.value.endpoint == "economic-indicators"


def test_decode_error_names_the_risk_premium_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A well-formed but wrongly shaped row on the risk-premium route is a decode error."""
    fixture_server.route("/market-risk-premium", [{"country": "Zimbabwe"}])
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.economics.market_risk_premium()
    assert raised.value.endpoint == "market-risk-premium"
