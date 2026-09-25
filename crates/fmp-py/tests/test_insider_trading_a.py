"""Runtime contract of ``client.insider_trading``: the latest feed and the trade search.

One test per argument shape routes the documented fixture body, calls the
method, and asserts the exact request target plus a few typed fields
(including the ``datetime.date`` ones). The expected targets are the ones the
Rust ``insider_trading_latest_search_endpoints.rs`` test pins. The reference
lookups, the beneficial-ownership filings, and the negative cases live in
``test_insider_trading_b.py``.
"""

import datetime
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.insider_trading import InsiderTrade, InsiderTradingNamespace

SPACED_SYMBOL = "BRK.B / Class A"
SPACED_SYMBOL_ENCODED = "BRK.B+%2F+Class+A"
REPORTING_CIK = "0001496686"
COMPANY_CIK = "0000320193"
SPACED_TRANSACTION_TYPE = "S-Sale / future"
SPACED_TRANSACTION_TYPE_ENCODED = "S-Sale+%2F+future"
U32_MAX = 4_294_967_295


def assert_trmk_award(row: Any) -> None:
    """Assert the shared ``InsiderTrade`` fixture row both endpoints return."""
    assert isinstance(row, InsiderTrade)
    assert row.symbol == "TRMK"
    assert row.filing_date == datetime.date(2026, 7, 30)
    assert row.transaction_date == datetime.date(2026, 7, 28)
    assert row.reporting_cik == "0001661867"
    assert row.company_cik == "0000036146"
    assert row.transaction_type == "A-Award"
    assert row.securities_owned == 62_959
    assert row.reporting_name == "Tate Granville Jr"
    assert row.type_of_owner == "officer: Secretary"
    assert row.acquisition_or_disposition == "A"
    assert row.direct_or_indirect == "D"
    assert row.form_type == "4"
    assert row.securities_transacted == 1_608
    assert isinstance(row.price, float)
    assert row.price == pytest.approx(0.0)
    assert row.security_name == "Common Stock"
    assert row.url.startswith("https://www.sec.gov/Archives/edgar/data/36146/")


def test_insider_trading_namespace_is_the_generated_type(client: Any) -> None:
    """``client.insider_trading`` is the generated flat namespace class."""
    assert isinstance(client.insider_trading, InsiderTradingNamespace)


def test_latest_trades_with_every_filter(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_trades`` encodes ``date``, ``page``, then ``limit`` and decodes the row."""
    fixture_server.route("/insider-trading/latest", load_fixture("latest_insider_trades.json"))
    rows = client.insider_trading.latest_trades(date="2026-01-27", page=0, limit=U32_MAX)

    assert fixture_server.requests[0].target == f"/insider-trading/latest?date=2026-01-27&page=0&limit={U32_MAX}"
    assert len(rows) == 1
    assert_trmk_award(rows[0])


def test_latest_trades_accepts_a_datetime_date(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_trades`` formats a ``datetime.date`` as ``YYYY-MM-DD`` on the wire."""
    fixture_server.route("/insider-trading/latest", load_fixture("latest_insider_trades.json"))
    rows = client.insider_trading.latest_trades(date=datetime.date(2026, 1, 27))

    assert fixture_server.requests[0].target == "/insider-trading/latest?date=2026-01-27"
    assert len(rows) == 1


def test_latest_trades_with_page_only(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_trades`` with only ``page`` omits ``date`` and ``limit`` from the query string."""
    fixture_server.route("/insider-trading/latest", load_fixture("latest_insider_trades.json"))
    rows = client.insider_trading.latest_trades(page=100)

    assert fixture_server.requests[0].target == "/insider-trading/latest?page=100"
    assert len(rows) == 1


def test_latest_trades_without_options_sends_the_bare_path(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_trades`` with no options requests the bare path and takes no positional argument."""
    fixture_server.route("/insider-trading/latest", load_fixture("latest_insider_trades.json"))
    rows = client.insider_trading.latest_trades()

    assert fixture_server.requests[0].target == "/insider-trading/latest"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1
    with pytest.raises(TypeError):
        client.insider_trading.latest_trades("2026-01-27")


def test_search_trades_with_every_filter(client: Any, fixture_server: FixtureServer) -> None:
    """``search_trades`` encodes the six filters in the documented order, form-encoding the spaced values."""
    fixture_server.route("/insider-trading/search", load_fixture("searched_insider_trades.json"))
    rows = client.insider_trading.search_trades(
        symbol=SPACED_SYMBOL,
        page=U32_MAX,
        limit=0,
        reporting_cik=REPORTING_CIK,
        company_cik=COMPANY_CIK,
        transaction_type=SPACED_TRANSACTION_TYPE,
    )

    assert fixture_server.requests[0].target == (
        f"/insider-trading/search?symbol={SPACED_SYMBOL_ENCODED}&page={U32_MAX}&limit=0"
        f"&reportingCik={REPORTING_CIK}&companyCik={COMPANY_CIK}"
        f"&transactionType={SPACED_TRANSACTION_TYPE_ENCODED}"
    )
    assert fixture_server.requests[0].query["symbol"] == [SPACED_SYMBOL]
    assert fixture_server.requests[0].query["transactionType"] == [SPACED_TRANSACTION_TYPE]
    assert len(rows) == 1
    assert_trmk_award(rows[0])


def test_search_trades_with_symbol_only(client: Any, fixture_server: FixtureServer) -> None:
    """``search_trades`` with only ``symbol`` sends a single parameter."""
    fixture_server.route("/insider-trading/search", load_fixture("searched_insider_trades.json"))
    rows = client.insider_trading.search_trades(symbol="AAPL")

    assert fixture_server.requests[0].target == "/insider-trading/search?symbol=AAPL"
    assert len(rows) == 1


def test_search_trades_with_cik_filters_only(client: Any, fixture_server: FixtureServer) -> None:
    """``search_trades`` keeps the leading zeros of both CIK filters and skips the unset ones."""
    fixture_server.route("/insider-trading/search", load_fixture("searched_insider_trades.json"))
    rows = client.insider_trading.search_trades(reporting_cik=REPORTING_CIK, company_cik=COMPANY_CIK)

    assert (
        fixture_server.requests[0].target
        == f"/insider-trading/search?reportingCik={REPORTING_CIK}&companyCik={COMPANY_CIK}"
    )
    assert len(rows) == 1


def test_search_trades_with_transaction_type_only(client: Any, fixture_server: FixtureServer) -> None:
    """``search_trades`` sends an open transaction-type code verbatim."""
    fixture_server.route("/insider-trading/search", load_fixture("searched_insider_trades.json"))
    rows = client.insider_trading.search_trades(transaction_type="A-Award")

    assert fixture_server.requests[0].target == "/insider-trading/search?transactionType=A-Award"
    assert len(rows) == 1


def test_search_trades_with_page_and_limit_only(client: Any, fixture_server: FixtureServer) -> None:
    """``search_trades`` encodes ``page`` before ``limit`` when only the paging is set."""
    fixture_server.route("/insider-trading/search", load_fixture("searched_insider_trades.json"))
    rows = client.insider_trading.search_trades(page=0, limit=100)

    assert fixture_server.requests[0].target == "/insider-trading/search?page=0&limit=100"
    assert len(rows) == 1


def test_search_trades_without_options_sends_the_bare_path(client: Any, fixture_server: FixtureServer) -> None:
    """``search_trades`` with no filters requests the bare path and takes no positional argument."""
    fixture_server.route("/insider-trading/search", load_fixture("searched_insider_trades.json"))
    rows = client.insider_trading.search_trades()

    assert fixture_server.requests[0].target == "/insider-trading/search"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1
    with pytest.raises(TypeError):
        client.insider_trading.search_trades("AAPL")


def test_search_and_latest_decode_the_same_row(client: Any, fixture_server: FixtureServer) -> None:
    """The latest feed and the search share one row model and decode the shared fixture identically."""
    fixture_server.route("/insider-trading/latest", load_fixture("latest_insider_trades.json"))
    fixture_server.route("/insider-trading/search", load_fixture("searched_insider_trades.json"))
    latest = client.insider_trading.latest_trades()
    searched = client.insider_trading.search_trades()

    assert [request.path for request in fixture_server.requests] == [
        "/insider-trading/latest",
        "/insider-trading/search",
    ]
    assert type(latest[0]) is type(searched[0])
    assert latest[0].filing_date == searched[0].filing_date
    assert latest[0].securities_owned == searched[0].securities_owned


def test_latest_trades_decode_fractional_share_quantities(client: Any, fixture_server: FixtureServer) -> None:
    """Issue #340: fractional and integral-float share quantities decode as ``float``."""
    fixture_server.route("/insider-trading/latest", load_fixture("latest_insider_trades_fractional_synthetic.json"))
    rows = client.insider_trading.latest_trades()

    assert len(rows) == 1
    assert isinstance(rows[0].securities_owned, float)
    assert rows[0].securities_owned == 62_959.5
    assert rows[0].securities_transacted == 1.0
