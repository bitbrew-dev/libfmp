"""Runtime contract of ``client.company``: profiles, notes, peers, workforce, market cap.

One test per method routes the documented fixture body, calls the method with
one argument shape, and asserts the exact request target plus a few typed
fields (including the ``datetime.date`` / ``datetime.datetime`` ones). The
expected targets are the ones the Rust ``company_endpoints.rs``,
``company_workforce_endpoints.rs``, and ``company_market_data_endpoints.rs``
tests pin. Share float, mergers and acquisitions, governance, and the
negative cases live in ``test_company_b.py``.
"""

import datetime
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.company import (
    CompanyNamespace,
    CompanyNote,
    CompanyProfile,
    DelistedCompany,
    EmployeeCount,
    MarketCapitalizationRecord,
    StockPeer,
)

SPACED_SYMBOL = "BRK.B / Class A"
SPACED_SYMBOL_ENCODED = "BRK.B+%2F+Class+A"
APPLE_CIK = "0000320193"
DATE_2026_07_30 = datetime.date(2026, 7, 30)


def test_company_namespace_is_the_generated_type(client: Any) -> None:
    """``client.company`` is the generated flat namespace class."""
    assert isinstance(client.company, CompanyNamespace)


def test_profile_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``profile`` form-encodes the ticker and decodes the documented profile row."""
    fixture_server.route("/profile", load_fixture("company_profile.json"))
    rows = client.company.profile(SPACED_SYMBOL)

    assert fixture_server.requests[0].target == f"/profile?symbol={SPACED_SYMBOL_ENCODED}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CompanyProfile)
    assert row.symbol == "AAPL"
    assert row.company_name == "Apple Inc."
    assert row.cik == APPLE_CIK
    assert row.ipo_date == datetime.date(1980, 12, 12)
    assert row.market_cap == 4_874_072_686_740
    assert row.price == pytest.approx(331.85501)
    assert row.full_time_employees == "166000"
    assert row.is_etf is False
    assert row.is_actively_trading is True


def test_profile_decodes_multiple_rows_with_large_integers(client: Any, fixture_server: FixtureServer) -> None:
    """``profile`` keeps row order; ``market_cap`` and ``volume`` are floats and round above ``2**53``."""
    fixture_server.route("/profile", load_fixture("company_profile_multiple.json"))
    rows = client.company.profile("AAPL")

    assert fixture_server.requests[0].target == "/profile?symbol=AAPL"
    assert len(rows) == 2
    assert [row.symbol for row in rows] == ["BIG", "AAPL"]
    assert isinstance(rows[0].volume, float)
    assert rows[0].volume == float(18_446_744_073_709_551_615)
    assert isinstance(rows[0].market_cap, float)
    assert rows[0].market_cap == float(9_007_199_254_740_993)
    assert rows[0].ipo_date == datetime.date(2026, 8, 27)
    assert rows[0].is_fund is True
    assert rows[1].ipo_date == datetime.date(1980, 12, 12)


def test_profile_by_cik_sends_the_cik_verbatim(client: Any, fixture_server: FixtureServer) -> None:
    """``profile_by_cik`` maps to ``/profile-cik`` and keeps the leading zeros."""
    fixture_server.route("/profile-cik", load_fixture("company_profile.json"))
    rows = client.company.profile_by_cik(APPLE_CIK)

    assert fixture_server.requests[0].target == f"/profile-cik?cik={APPLE_CIK}"
    assert len(rows) == 1
    assert isinstance(rows[0], CompanyProfile)
    assert rows[0].cik == APPLE_CIK
    assert rows[0].exchange == "NASDAQ"
    assert rows[0].beta == pytest.approx(1.097)


def test_notes_takes_one_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``notes`` maps to ``/company-notes`` with the symbol alone."""
    fixture_server.route("/company-notes", load_fixture("company_note.json"))
    rows = client.company.notes("AAPL")

    assert fixture_server.requests[0].target == "/company-notes?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CompanyNote)
    assert row.cik == APPLE_CIK
    assert row.symbol == "AAPL"
    assert row.title == "0.000% Notes due 2025"
    assert row.exchange == "NASDAQ"


def test_stock_peers_encodes_a_caret_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``stock_peers`` percent-encodes ``^`` and decodes the peer row."""
    fixture_server.route("/stock-peers", load_fixture("stock_peer.json"))
    rows = client.company.stock_peers("^VIX")

    assert fixture_server.requests[0].target == "/stock-peers?symbol=%5EVIX"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockPeer)
    assert row.symbol == "GOOGL"
    assert row.company_name == "Alphabet Inc."
    assert row.price == pytest.approx(333.84)
    assert row.market_cap == 4_040_168_831_718


def test_delisted_companies_with_page_and_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``delisted_companies`` encodes ``page`` then ``limit`` and decodes both dates."""
    fixture_server.route("/delisted-companies", load_fixture("company_delisted.json"))
    rows = client.company.delisted_companies(page=0, limit=101)

    assert fixture_server.requests[0].target == "/delisted-companies?page=0&limit=101"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, DelistedCompany)
    assert row.symbol == "CCIX"
    assert row.company_name == "Churchill Capital Corp IX Ordinary Shares"
    assert row.exchange == "NASDAQ"
    assert row.ipo_date == datetime.date(2007, 3, 1)
    assert row.delisted_date == datetime.date(2026, 7, 28)


def test_delisted_companies_with_page_only(client: Any, fixture_server: FixtureServer) -> None:
    """``delisted_companies`` sends ``page=0`` alone; ``0`` is emitted, not dropped."""
    fixture_server.route("/delisted-companies", load_fixture("company_delisted.json"))
    rows = client.company.delisted_companies(page=0)

    assert fixture_server.requests[0].target == "/delisted-companies?page=0"
    assert len(rows) == 1


def test_delisted_companies_without_options_sends_the_bare_path(client: Any, fixture_server: FixtureServer) -> None:
    """``delisted_companies`` with nothing set requests the bare path and decodes an empty body."""
    fixture_server.route("/delisted-companies", load_fixture("company_empty.json"))
    rows = client.company.delisted_companies()

    assert fixture_server.requests[0].target == "/delisted-companies"
    assert fixture_server.requests[0].raw_query == ""
    assert rows == []


def test_employee_count_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``employee_count`` form-encodes the ticker and decodes the SEC filing fields."""
    fixture_server.route("/employee-count", load_fixture("company_employee_count.json"))
    rows = client.company.employee_count(SPACED_SYMBOL)

    assert fixture_server.requests[0].target == f"/employee-count?symbol={SPACED_SYMBOL_ENCODED}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EmployeeCount)
    assert row.symbol == "AAPL"
    assert row.cik == APPLE_CIK
    assert row.employee_count == 166_000
    assert row.form_type == "10-K"
    assert row.filing_date == datetime.date(2025, 10, 31)
    assert row.period_of_report == datetime.date(2025, 9, 27)
    assert row.acceptance_time == datetime.datetime(2025, 10, 31, 6, 1, 26)


def test_employee_count_with_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``employee_count`` encodes ``symbol`` then ``limit``."""
    fixture_server.route("/employee-count", load_fixture("company_employee_count.json"))
    rows = client.company.employee_count("AAPL", limit=10_001)

    assert fixture_server.requests[0].target == "/employee-count?symbol=AAPL&limit=10001"
    assert len(rows) == 1
    assert rows[0].source.startswith("https://www.sec.gov/Archives/edgar/data/320193/")


def test_historical_employee_count_without_options(client: Any, fixture_server: FixtureServer) -> None:
    """``historical_employee_count`` sends only the symbol and shares the ``EmployeeCount`` model."""
    fixture_server.route("/historical-employee-count", load_fixture("company_employee_count.json"))
    rows = client.company.historical_employee_count("AAPL")

    assert fixture_server.requests[0].target == "/historical-employee-count?symbol=AAPL"
    assert len(rows) == 1
    assert isinstance(rows[0], EmployeeCount)
    assert rows[0].employee_count == 166_000
    assert rows[0].acceptance_time == datetime.datetime(2025, 10, 31, 6, 1, 26)


def test_historical_employee_count_with_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``historical_employee_count`` encodes ``symbol`` then ``limit``."""
    fixture_server.route("/historical-employee-count", load_fixture("company_employee_count.json"))
    rows = client.company.historical_employee_count("AAPL", limit=10_001)

    assert fixture_server.requests[0].target == "/historical-employee-count?symbol=AAPL&limit=10001"
    assert len(rows) == 1
    assert rows[0].filing_date == datetime.date(2025, 10, 31)


def test_market_capitalization_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``market_capitalization`` form-encodes the ticker and decodes the dated record."""
    fixture_server.route("/market-capitalization", load_fixture("company_market_capitalization.json"))
    rows = client.company.market_capitalization(SPACED_SYMBOL)

    assert fixture_server.requests[0].target == f"/market-capitalization?symbol={SPACED_SYMBOL_ENCODED}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, MarketCapitalizationRecord)
    assert row.symbol == "AAPL"
    assert row.date == DATE_2026_07_30
    assert row.market_cap == 4_874_072_686_740


def test_market_capitalization_batch_joins_and_encodes_symbols(client: Any, fixture_server: FixtureServer) -> None:
    """``market_capitalization_batch`` comma-joins the list and percent-encodes each ticker."""
    fixture_server.route("/market-capitalization-batch", load_fixture("company_market_capitalization.json"))
    rows = client.company.market_capitalization_batch(["AAPL", "^VIX", "000001.SZ"])

    assert fixture_server.requests[0].target == "/market-capitalization-batch?symbols=AAPL%2C%5EVIX%2C000001.SZ"
    assert len(rows) == 1
    assert isinstance(rows[0], MarketCapitalizationRecord)
    assert rows[0].date == DATE_2026_07_30
    assert rows[0].market_cap == 4_874_072_686_740


def test_market_capitalization_batch_accepts_a_bare_string(client: Any, fixture_server: FixtureServer) -> None:
    """A bare ``str`` is one ticker, never split on commas."""
    fixture_server.route("/market-capitalization-batch", load_fixture("company_market_capitalization.json"))
    rows = client.company.market_capitalization_batch("AAPL")

    assert fixture_server.requests[0].target == "/market-capitalization-batch?symbols=AAPL"
    assert len(rows) == 1


def test_historical_market_capitalization_without_options(client: Any, fixture_server: FixtureServer) -> None:
    """``historical_market_capitalization`` sends only the symbol when nothing else is set."""
    fixture_server.route(
        "/historical-market-capitalization", load_fixture("company_historical_market_capitalization.json")
    )
    rows = client.company.historical_market_capitalization("AAPL")

    assert fixture_server.requests[0].target == "/historical-market-capitalization?symbol=AAPL"
    assert "limit=" not in fixture_server.requests[0].raw_query
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, MarketCapitalizationRecord)
    assert row.symbol == "AAPL"
    assert row.date == DATE_2026_07_30
    assert row.market_cap == 4_879_177_245_542


def test_historical_market_capitalization_with_from_only(client: Any, fixture_server: FixtureServer) -> None:
    """``from_`` is sent as ``from`` on the wire, independently of ``to``."""
    fixture_server.route(
        "/historical-market-capitalization", load_fixture("company_historical_market_capitalization.json")
    )
    rows = client.company.historical_market_capitalization("AAPL", from_=datetime.date(2026, 4, 16))

    assert fixture_server.requests[0].target == "/historical-market-capitalization?symbol=AAPL&from=2026-04-16"
    assert fixture_server.requests[0].query == {"symbol": ["AAPL"], "from": ["2026-04-16"]}
    assert len(rows) == 1


def test_historical_market_capitalization_with_to_only(client: Any, fixture_server: FixtureServer) -> None:
    """``to`` accepts an ISO string and is sent alone when ``from_`` is omitted."""
    fixture_server.route(
        "/historical-market-capitalization", load_fixture("company_historical_market_capitalization.json")
    )
    rows = client.company.historical_market_capitalization("AAPL", to="2026-07-16")

    assert fixture_server.requests[0].target == "/historical-market-capitalization?symbol=AAPL&to=2026-07-16"
    assert len(rows) == 1


def test_historical_market_capitalization_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """Encoding order is ``symbol``, ``limit``, ``from``, ``to``; the dates are not range-checked."""
    fixture_server.route(
        "/historical-market-capitalization", load_fixture("company_historical_market_capitalization.json")
    )
    rows = client.company.historical_market_capitalization(
        "AAPL", limit=5001, from_="2026-07-16", to=datetime.date(2026, 4, 16)
    )

    assert (
        fixture_server.requests[0].target
        == "/historical-market-capitalization?symbol=AAPL&limit=5001&from=2026-07-16&to=2026-04-16"
    )
    assert len(rows) == 1
    assert rows[0].date == DATE_2026_07_30
