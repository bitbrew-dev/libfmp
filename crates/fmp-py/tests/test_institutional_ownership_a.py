"""Runtime contract of ``client.institutional_ownership``: filings, extract, dates, analytics.

One test per method routes the documented fixture body, calls the method with
one argument shape, and asserts the exact request target plus a few typed
fields (including the ``datetime.date`` / ``datetime.datetime`` ones). The
expected targets are the ones the Rust ``institutional_ownership_endpoints.rs``
and ``institutional_holder_analytics_endpoints.rs`` tests pin. The holder
summaries, the position and industry summaries, and the negative cases live in
``test_institutional_ownership_b.py``.
"""

import datetime
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.institutional_ownership import (
    Form13fFilingDate,
    InstitutionalHolderAnalytics,
    InstitutionalHolding,
    InstitutionalOwnershipFiling,
    InstitutionalOwnershipNamespace,
)

HOLDER_CIK = "0001388838"
BERKSHIRE_CIK = "0001067983"
YEAR_2023 = 2023
Q3 = 3
SPACED_SYMBOL = "BRK.B / Class A"
SPACED_SYMBOL_ENCODED = "BRK.B+%2F+Class+A"


def test_institutional_ownership_namespace_is_the_generated_type(client: Any) -> None:
    """``client.institutional_ownership`` is the generated flat namespace class."""
    assert isinstance(client.institutional_ownership, InstitutionalOwnershipNamespace)


def test_latest_filings_with_page_and_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_filings`` encodes ``page`` before ``limit`` and decodes both datetime fields."""
    fixture_server.route(
        "/institutional-ownership/latest", load_fixture("latest_institutional_ownership_filings.json")
    )
    rows = client.institutional_ownership.latest_filings(page=0, limit=100)

    assert fixture_server.requests[0].target == "/institutional-ownership/latest?page=0&limit=100"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, InstitutionalOwnershipFiling)
    assert row.cik == "0001803005"
    assert row.name == "WEALTH ADVISORS OF IOWA, LLC"
    assert row.form_type == "13F-HR"
    assert row.date == datetime.date(2026, 6, 30)
    assert row.filing_date == datetime.datetime(2026, 7, 30, 0, 0, 0)
    assert row.accepted_date == datetime.datetime(2026, 7, 30, 13, 14, 23)
    assert row.final_link.startswith("https://www.sec.gov/Archives/edgar/data/1803005/")


def test_latest_filings_with_page_only(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_filings`` with only ``page`` omits ``limit`` from the query string."""
    fixture_server.route(
        "/institutional-ownership/latest", load_fixture("latest_institutional_ownership_filings.json")
    )
    rows = client.institutional_ownership.latest_filings(page=100)

    assert fixture_server.requests[0].target == "/institutional-ownership/latest?page=100"
    assert len(rows) == 1


def test_latest_filings_without_options_sends_the_bare_path(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_filings`` with no options requests the bare path and takes no positional argument."""
    fixture_server.route(
        "/institutional-ownership/latest", load_fixture("latest_institutional_ownership_filings.json")
    )
    rows = client.institutional_ownership.latest_filings()

    assert fixture_server.requests[0].target == "/institutional-ownership/latest"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1
    with pytest.raises(TypeError):
        client.institutional_ownership.latest_filings(0)


def test_extract_encodes_cik_year_then_quarter(client: Any, fixture_server: FixtureServer) -> None:
    """``extract`` sends the leading-zero CIK, the year, and the numeric quarter in order."""
    fixture_server.route("/institutional-ownership/extract", load_fixture("institutional_ownership_extract.json"))
    rows = client.institutional_ownership.extract(HOLDER_CIK, YEAR_2023, Q3)

    assert fixture_server.requests[0].target == f"/institutional-ownership/extract?cik={HOLDER_CIK}&year=2023&quarter=3"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, InstitutionalHolding)
    assert row.cik == HOLDER_CIK
    assert row.symbol == "CHRD"
    assert row.security_cusip == "674215207"
    assert row.name_of_issuer == "CHORD ENERGY CORPORATION"
    assert row.shares == 13_280
    assert row.value == 2_152_290
    assert row.put_call_share == ""
    assert row.date == datetime.date(2023, 9, 30)
    assert row.filing_date == datetime.date(2023, 11, 13)
    assert row.accepted_date == datetime.date(2023, 11, 13)


def test_extract_accepts_keyword_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``extract`` takes its required arguments by keyword as well as by position."""
    fixture_server.route("/institutional-ownership/extract", load_fixture("institutional_ownership_extract.json"))
    rows = client.institutional_ownership.extract(cik=HOLDER_CIK, year=YEAR_2023, quarter=Q3)

    assert fixture_server.requests[0].query == {"cik": [HOLDER_CIK], "year": ["2023"], "quarter": ["3"]}
    assert len(rows) == 1


def test_form_13f_filing_dates_sends_the_cik_verbatim(client: Any, fixture_server: FixtureServer) -> None:
    """``form_13f_filing_dates`` maps to ``/institutional-ownership/dates`` and keeps the leading zeros."""
    fixture_server.route("/institutional-ownership/dates", load_fixture("form_13f_filing_dates.json"))
    rows = client.institutional_ownership.form_13f_filing_dates(BERKSHIRE_CIK)

    assert fixture_server.requests[0].target == f"/institutional-ownership/dates?cik={BERKSHIRE_CIK}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, Form13fFilingDate)
    assert row.date == datetime.date(2026, 3, 31)
    assert row.year == 2026
    assert row.quarter == 1


def test_holder_analytics_with_page_and_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``holder_analytics`` encodes symbol, year, quarter, page, then limit, and decodes the row."""
    fixture_server.route(
        "/institutional-ownership/extract-analytics/holder", load_fixture("institutional_holder_analytics.json")
    )
    rows = client.institutional_ownership.holder_analytics("AAPL", YEAR_2023, Q3, page=0, limit=10)

    assert (
        fixture_server.requests[0].target
        == "/institutional-ownership/extract-analytics/holder?symbol=AAPL&year=2023&quarter=3&page=0&limit=10"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, InstitutionalHolderAnalytics)
    assert row.cik == "0000102909"
    assert row.investor_name == "VANGUARD GROUP INC"
    assert row.security_cusip == "037833100"
    assert row.date == datetime.date(2023, 9, 30)
    assert row.filing_date == datetime.date(2023, 12, 18)
    assert row.first_added == datetime.date(2005, 3, 31)
    assert row.change_in_performance == -67_750_129_670
    assert row.market_value == 222_572_509_140
    assert row.avg_price_paid == pytest.approx(20.65)
    assert row.weight == pytest.approx(5.4673)
    assert row.holding_period == 75
    assert row.is_new is False
    assert row.is_sold_out is False
    assert row.is_counted_for_performance is True


def test_holder_analytics_with_limit_only(client: Any, fixture_server: FixtureServer) -> None:
    """``holder_analytics`` with only ``limit`` omits ``page`` and accepts a zero limit."""
    fixture_server.route(
        "/institutional-ownership/extract-analytics/holder", load_fixture("institutional_holder_analytics.json")
    )
    rows = client.institutional_ownership.holder_analytics("AAPL", YEAR_2023, Q3, limit=0)

    assert (
        fixture_server.requests[0].target
        == "/institutional-ownership/extract-analytics/holder?symbol=AAPL&year=2023&quarter=3&limit=0"
    )
    assert len(rows) == 1


def test_holder_analytics_without_options_encodes_a_spaced_symbol(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``holder_analytics`` form-encodes the ticker and sends only the required parameters."""
    fixture_server.route(
        "/institutional-ownership/extract-analytics/holder", load_fixture("institutional_holder_analytics.json")
    )
    rows = client.institutional_ownership.holder_analytics(SPACED_SYMBOL, YEAR_2023, Q3)

    assert (
        fixture_server.requests[0].target
        == f"/institutional-ownership/extract-analytics/holder?symbol={SPACED_SYMBOL_ENCODED}&year=2023&quarter=3"
    )
    assert fixture_server.requests[0].query["symbol"] == [SPACED_SYMBOL]
    assert len(rows) == 1


def test_holder_analytics_accepts_the_u32_boundary(client: Any, fixture_server: FixtureServer) -> None:
    """``holder_analytics`` passes ``2**32 - 1`` for both ``page`` and ``limit`` unchanged."""
    fixture_server.route(
        "/institutional-ownership/extract-analytics/holder", load_fixture("institutional_holder_analytics.json")
    )
    client.institutional_ownership.holder_analytics("AAPL", YEAR_2023, Q3, page=4_294_967_295, limit=4_294_967_295)

    assert fixture_server.requests[0].query["page"] == ["4294967295"]
    assert fixture_server.requests[0].query["limit"] == ["4294967295"]


def test_extract_decodes_fractional_shares_and_value(client: Any, fixture_server: FixtureServer) -> None:
    """Issue #340: fractional shares and an integral-float value decode as ``float``."""
    fixture_server.route(
        "/institutional-ownership/extract", load_fixture("institutional_ownership_extract_fractional_synthetic.json")
    )
    rows = client.institutional_ownership.extract(HOLDER_CIK, YEAR_2023, Q3)

    assert len(rows) == 1
    assert isinstance(rows[0].shares, float)
    assert rows[0].shares == 13_280.5
    assert rows[0].value == 1.0
