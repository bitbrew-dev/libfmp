"""Runtime contract of ``client.calendar`` for the IPO methods and the argument rules.

Covers ``ipos_calendar``, ``ipos_disclosure``, and ``ipos_prospectus`` (the
``date_query!`` shape with independent ``from_`` / ``to``), the keyword
rename (``from`` on the wire, ``from_`` in Python), and one negative case per
argument-kind family. The expected targets are the ones the Rust
``calendar_ipos_endpoints.rs`` test pins.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.calendar import IpoCalendarEvent, IpoDisclosure, IpoProspectus

FROM_2026_03_06 = datetime.date(2026, 3, 6)
TO_2026_06_06 = datetime.date(2026, 6, 6)


def test_ipos_calendar_with_both_dates(client: Any, fixture_server: FixtureServer) -> None:
    """``ipos_calendar`` encodes ``from`` then ``to`` and exposes the dynamic JSON fields."""
    fixture_server.route("/ipos-calendar", load_fixture("ipos_calendar.json"))
    rows = client.calendar.ipos_calendar(from_="2026-03-06", to=TO_2026_06_06)

    assert fixture_server.requests[0].target == "/ipos-calendar?from=2026-03-06&to=2026-06-06"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, IpoCalendarEvent)
    assert row.symbol == "IMC"
    assert row.date == datetime.date(2026, 7, 29)
    assert row.daa == "2026-07-29T04:00:00.000Z"
    assert row.company == "IMC Rare Earths Ltd"
    assert row.exchange == "NYSE"
    assert row.actions == "Priced"
    assert (row.shares, row.price_range, row.market_cap) == (None, None, None)


def test_ipos_calendar_without_dates_has_no_query_string(client: Any, fixture_server: FixtureServer) -> None:
    """``ipos_calendar`` with nothing set requests the bare path."""
    fixture_server.route("/ipos-calendar", load_fixture("ipos_calendar.json"))
    rows = client.calendar.ipos_calendar()

    assert fixture_server.requests[0].target == "/ipos-calendar"
    assert len(rows) == 1


def test_ipos_disclosure_with_to_only(client: Any, fixture_server: FixtureServer) -> None:
    """``ipos_disclosure`` sends ``to`` alone when ``from_`` is omitted."""
    fixture_server.route("/ipos-disclosure", load_fixture("ipos_disclosure.json"))
    rows = client.calendar.ipos_disclosure(to="2026-06-06")

    assert fixture_server.requests[0].target == "/ipos-disclosure?to=2026-06-06"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, IpoDisclosure)
    assert row.symbol == "QTJA"
    assert row.filing_date == datetime.date(2026, 7, 30)
    assert row.accepted_date == datetime.date(2026, 7, 30)
    assert row.effectiveness_date == datetime.date(2026, 7, 30)
    assert row.cik == "0001415726"
    assert row.form == "CERT"
    assert row.url.startswith("https://www.sec.gov/Archives/edgar/data/1415726/")


def test_ipos_disclosure_with_both_dates(client: Any, fixture_server: FixtureServer) -> None:
    """``ipos_disclosure`` accepts ``datetime.date`` for both bounds."""
    fixture_server.route("/ipos-disclosure", load_fixture("ipos_disclosure.json"))
    rows = client.calendar.ipos_disclosure(from_=FROM_2026_03_06, to=TO_2026_06_06)

    assert fixture_server.requests[0].target == "/ipos-disclosure?from=2026-03-06&to=2026-06-06"
    assert len(rows) == 1


def test_ipos_prospectus_with_from_only(client: Any, fixture_server: FixtureServer) -> None:
    """``ipos_prospectus`` sends ``from`` alone and decodes the offering values as floats."""
    fixture_server.route("/ipos-prospectus", load_fixture("ipos_prospectus.json"))
    rows = client.calendar.ipos_prospectus(from_=FROM_2026_03_06)

    assert fixture_server.requests[0].target == "/ipos-prospectus?from=2026-03-06"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, IpoProspectus)
    assert row.symbol == "FTW-WT"
    assert row.ipo_date == datetime.date(2026, 7, 28)
    assert row.accepted_date == datetime.date(2026, 7, 29)
    assert row.filing_date == datetime.date(2026, 7, 30)
    assert row.cik == "0002083125"
    assert row.form == "S-1"
    assert row.price_public_per_share == pytest.approx(1.0)
    assert row.price_public_total == pytest.approx(434.0)
    assert row.discounts_and_commissions_per_share == pytest.approx(0.0)
    assert row.discounts_and_commissions_total == pytest.approx(82_251.0)
    assert row.proceeds_before_expenses_total == pytest.approx(82_251.0)


def test_ipos_prospectus_with_both_dates(client: Any, fixture_server: FixtureServer) -> None:
    """``ipos_prospectus`` encodes both ISO strings in ``from``, ``to`` order."""
    fixture_server.route("/ipos-prospectus", load_fixture("ipos_prospectus.json"))
    rows = client.calendar.ipos_prospectus(from_="2026-03-06", to="2026-06-06")

    assert fixture_server.requests[0].target == "/ipos-prospectus?from=2026-03-06&to=2026-06-06"
    assert len(rows) == 1


def test_from_keyword_is_renamed_but_the_wire_key_is_not(client: Any, fixture_server: FixtureServer) -> None:
    """The Python keyword is ``from_``; ``from`` is rejected as a keyword and the wire key stays ``from``."""
    fixture_server.route("/ipos-calendar", load_fixture("ipos_calendar.json"))
    with pytest.raises(TypeError, match="unexpected keyword argument 'from'"):
        client.calendar.ipos_calendar(**{"from": "2026-03-06"})
    assert fixture_server.requests == []

    client.calendar.ipos_calendar(from_="2026-03-06")
    assert fixture_server.requests[0].query == {"from": ["2026-03-06"]}
    assert "from_" not in fixture_server.requests[0].raw_query


@pytest.mark.parametrize(
    ("keyword", "value"),
    [("from_", "06/03/2026"), ("to", "2026-13-01"), ("from_", ""), ("to", "2026-6-6")],
)
def test_non_iso_date_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, keyword: str, value: str
) -> None:
    """A date string that is not ``YYYY-MM-DD`` fails locally, prefixed with the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.calendar.ipos_calendar(**{keyword: value})
    error = raised.value
    assert str(error) == f"{keyword}: value must be a valid YYYY-MM-DD date"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_negative_page_names_the_argument(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A negative ``page`` is rejected locally with the keyword as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.calendar.dividends_calendar(page=-1)
    assert str(raised.value) == "page: must be an integer from 0 through 4294967295"
    assert fixture_server.requests == []


def test_negative_limit_names_the_argument(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A negative ``limit`` is rejected locally with the keyword as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.calendar.stock_splits("AAPL", limit=-1)
    assert str(raised.value).startswith("limit: ")
    assert fixture_server.requests == []


def test_blank_symbol_names_the_argument(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A whitespace-only ``symbol`` is rejected locally with the keyword as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.calendar.earnings("  ")
    assert str(raised.value) == "symbol: value must not be empty or whitespace-only"
    assert fixture_server.requests == []


def test_non_bool_report_time_flag_is_a_type_error(client: Any, fixture_server: FixtureServer) -> None:
    """``include_report_times`` is a bare ``bool``; a string is a shape error, not a validation error."""
    with pytest.raises(TypeError):
        client.calendar.earnings("AAPL", include_report_times="yes")
    with pytest.raises(TypeError):
        client.calendar.earnings_calendar(include_report_times=1)
    assert fixture_server.requests == []


def test_status_error_carries_the_calendar_endpoint_id(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on a flat calendar method maps to ``FmpStatusError``."""
    fixture_server.route("/splits-calendar", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.calendar.stock_splits_calendar(page=0)
    error = raised.value
    assert error.endpoint == "splits-calendar"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_decode_error_on_a_non_json_body(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A body that is not JSON maps to ``FmpDecodeError`` rather than a bare exception."""
    fixture_server.route("/dividends", b"not-json")
    with pytest.raises(errors.FmpDecodeError):
        client.calendar.dividends("AAPL")
