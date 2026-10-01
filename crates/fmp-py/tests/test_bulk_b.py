"""Runtime contract of ``client.bulk`` for the statement and end-of-day methods.

Covers the six ``BulkStatementQuery`` methods (``year`` then ``period``, both
required) and ``eod`` (one required ``date``), plus the local validation rules
for every argument kind the domain uses. The expected targets are the ones the
Rust ``bulk_income_endpoints.rs``, ``bulk_balance_endpoints.rs``, and
``bulk_cash_eod_endpoints.rs`` tests pin.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import CSV_CONTENT_TYPE, FixtureServer, load_csv_fixture
from fmp.bulk.balance import BulkBalanceSheetStatement, BulkBalanceSheetStatementGrowth
from fmp.bulk.cash_flow import BulkCashFlowStatement, BulkCashFlowStatementGrowth
from fmp.bulk.eod import BulkEodBar
from fmp.bulk.income import BulkIncomeStatement, BulkIncomeStatementGrowth

DATE_2024_10_22 = datetime.date(2024, 10, 22)


def route_csv(fixture_server: FixtureServer, path: str, fixture: str) -> None:
    """Serve the shared CSV fixture on ``path`` as ``text/csv``, as the bulk routes answer."""
    fixture_server.route(path, load_csv_fixture(fixture), content_type=CSV_CONTENT_TYPE)


def test_income_statements_encodes_year_then_period(client: Any, fixture_server: FixtureServer) -> None:
    """``income_statements`` sends ``year`` before ``period`` and decodes the datetime column."""
    route_csv(fixture_server, "/income-statement-bulk", "bulk_income_statements.csv")
    rows = client.bulk.income_statements(2026, "Q1")

    assert fixture_server.requests[0].target == "/income-statement-bulk?year=2026&period=Q1"
    assert len(rows) == 3
    row = rows[0]
    assert isinstance(row, BulkIncomeStatement)
    assert row.symbol == "000001.SZ"
    assert row.date == datetime.date(2024, 12, 31)
    assert row.filing_date == datetime.date(2024, 12, 31)
    assert row.accepted_date == datetime.datetime(2024, 12, 31, 0, 0, 0)
    assert (row.fiscal_year, row.period) == ("2024", "FY")
    assert row.revenue == "251641000000"
    assert row.net_income == "44508000000"
    assert row.eps == "2.15"
    assert (rows[2].symbol, rows[2].eps, rows[2].weighted_average_shs_out_dil) == ("OASMY", None, None)


def test_income_statement_growth_with_keyword_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``income_statement_growth`` accepts both constructor arguments by keyword."""
    route_csv(fixture_server, "/income-statement-growth-bulk", "bulk_income_statement_growth.csv")
    rows = client.bulk.income_statement_growth(period="Q1", year=2026)

    assert fixture_server.requests[0].target == "/income-statement-growth-bulk?year=2026&period=Q1"
    assert len(rows) == 2
    row = rows[0]
    assert isinstance(row, BulkIncomeStatementGrowth)
    assert row.symbol == "000001.SZ"
    assert row.date == datetime.date(2024, 12, 31)
    assert (row.fiscal_year, row.period) == ("2024", "FY")
    assert row.growth_revenue == "-0.08220846812871789"
    assert row.growth_net_income == "-0.04191152728446884"
    assert row.growth_eps == "-0.04444444444444448"


def test_balance_sheet_statements_with_the_full_year_period(client: Any, fixture_server: FixtureServer) -> None:
    """``balance_sheet_statements`` accepts ``FY`` and decodes the mid-day accepted timestamp."""
    route_csv(fixture_server, "/balance-sheet-statement-bulk", "bulk_balance_sheet_statements.csv")
    rows = client.bulk.balance_sheet_statements(2025, "FY")

    assert fixture_server.requests[0].target == "/balance-sheet-statement-bulk?year=2025&period=FY"
    assert len(rows) == 2
    row = rows[0]
    assert isinstance(row, BulkBalanceSheetStatement)
    assert row.symbol == "000001.SZ"
    assert row.date == datetime.date(2024, 12, 31)
    assert row.filing_date == datetime.date(2024, 12, 31)
    assert row.accepted_date == datetime.datetime(2024, 12, 31, 0, 0, 0)
    assert row.total_assets == "5769270000000"
    assert row.cash_and_cash_equivalents == "680935000000"
    assert row.total_debt == "756251000000"


def test_balance_sheet_statement_growth_is_case_insensitive_on_period(
    client: Any, fixture_server: FixtureServer
) -> None:
    """A lower-case fiscal period is normalised to the documented upper-case spelling."""
    route_csv(fixture_server, "/balance-sheet-statement-growth-bulk", "bulk_balance_sheet_statement_growth.csv")
    rows = client.bulk.balance_sheet_statement_growth(2026, "q4")

    assert fixture_server.requests[0].target == "/balance-sheet-statement-growth-bulk?year=2026&period=Q4"
    assert len(rows) == 2
    row = rows[0]
    assert isinstance(row, BulkBalanceSheetStatementGrowth)
    assert row.symbol == "000001.SZ"
    assert row.date == datetime.date(2024, 12, 31)
    assert row.growth_total_assets == "0.032602509058340653"
    assert row.growth_cash_and_cash_equivalents == "0.13745663137040692"
    assert row.growth_net_debt == "-0.5927764260610976"


def test_cash_flow_statements_encodes_year_then_period(client: Any, fixture_server: FixtureServer) -> None:
    """``cash_flow_statements`` maps to ``/cash-flow-statement-bulk`` in documented order."""
    route_csv(fixture_server, "/cash-flow-statement-bulk", "bulk_cash_flow_statements.csv")
    rows = client.bulk.cash_flow_statements(2026, "Q2")

    assert fixture_server.requests[0].target == "/cash-flow-statement-bulk?year=2026&period=Q2"
    assert len(rows) == 2
    row = rows[0]
    assert isinstance(row, BulkCashFlowStatement)
    assert row.symbol == "000001.SZ"
    assert row.date == datetime.date(2024, 3, 31)
    assert row.accepted_date == datetime.datetime(2024, 3, 30, 20, 0, 0)
    assert row.net_income == "14932000000"
    assert row.free_cash_flow == "-21623000000"
    assert row.capital_expenditure == "-241000000"


def test_cash_flow_statement_growth_encodes_year_then_period(client: Any, fixture_server: FixtureServer) -> None:
    """``cash_flow_statement_growth`` maps to ``/cash-flow-statement-growth-bulk``."""
    route_csv(fixture_server, "/cash-flow-statement-growth-bulk", "bulk_cash_flow_statement_growth.csv")
    rows = client.bulk.cash_flow_statement_growth(2026, "Q3")

    assert fixture_server.requests[0].target == "/cash-flow-statement-growth-bulk?year=2026&period=Q3"
    assert len(rows) == 2
    row = rows[0]
    assert isinstance(row, BulkCashFlowStatementGrowth)
    assert row.symbol == "000001.SZ"
    assert row.date == datetime.date(2024, 12, 31)
    assert (row.fiscal_year, row.period) == ("2024", "FY")
    assert row.growth_net_income == "-0.04191152728446884"
    assert row.growth_free_cash_flow == "-0.3179880266323505"
    assert row.growth_capital_expenditure == "0.22868217054263565"


def test_eod_accepts_a_date_object(client: Any, fixture_server: FixtureServer) -> None:
    """``eod`` encodes a ``datetime.date`` as the ISO ``date`` parameter."""
    route_csv(fixture_server, "/eod-bulk", "bulk_eod.csv")
    rows = client.bulk.eod(DATE_2024_10_22)

    assert fixture_server.requests[0].target == "/eod-bulk?date=2024-10-22"
    assert len(rows) == 2
    row = rows[0]
    assert isinstance(row, BulkEodBar)
    assert row.symbol == "PCFD.SG"
    assert row.date == datetime.date(2025, 6, 2)
    assert row.open == "7.35"
    assert row.close == "19.71"
    assert row.adj_close == "19.71"
    assert row.volume == "0"


def test_eod_accepts_an_iso_string(client: Any, fixture_server: FixtureServer) -> None:
    """``eod`` also takes the date as an ISO string, by keyword."""
    route_csv(fixture_server, "/eod-bulk", "bulk_eod.csv")
    rows = client.bulk.eod(date="2024-10-22")

    assert fixture_server.requests[0].target == "/eod-bulk?date=2024-10-22"
    assert rows[1].date == datetime.date(2025, 6, 2)


def test_statement_arguments_are_required(client: Any, fixture_server: FixtureServer) -> None:
    """Both constructor arguments are positional-or-keyword and neither has a default."""
    with pytest.raises(TypeError):
        client.bulk.income_statements(2026)
    with pytest.raises(TypeError):
        client.bulk.eod()
    assert fixture_server.requests == []


@pytest.mark.parametrize("period", ["annual", "quarter", "Q5", ""])
def test_invalid_fiscal_period_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, period: str
) -> None:
    """Retrieval-frequency spellings are not fiscal periods here, unlike ``client.statements``."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.bulk.income_statements(2026, period)
    error = raised.value
    assert str(error) == "period: fiscal period must be one of Q1, Q2, Q3, Q4, FY"
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize("year", [-1, 4_294_967_296])
def test_out_of_range_year_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, year: int
) -> None:
    """A year outside the provider's unsigned 32-bit range is rejected locally."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.bulk.earnings_surprises(year)
    assert str(raised.value) == "year: must be an integer from 0 through 4294967295"
    with pytest.raises(errors.FmpValidationError) as raised:
        client.bulk.cash_flow_statements(year, "FY")
    assert str(raised.value).startswith("year: ")
    assert fixture_server.requests == []


@pytest.mark.parametrize("part", ["", "   ", "0\n1"])
def test_blank_or_control_part_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, part: str
) -> None:
    """An empty, whitespace-only, or control-character partition is rejected locally."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.bulk.company_profiles(part)
    assert str(raised.value).startswith("part: ")
    with pytest.raises(errors.FmpValidationError) as raised:
        client.bulk.etf_holdings(part)
    assert str(raised.value).startswith("part: ")
    assert fixture_server.requests == []


@pytest.mark.parametrize("date", ["2024-13-01", "22/10/2024", ""])
def test_malformed_date_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, date: str
) -> None:
    """A date string that is not ``YYYY-MM-DD`` is rejected before any request."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.bulk.eod(date)
    assert str(raised.value).startswith("date: ")
    assert raised.value.category == "validation"
    assert fixture_server.requests == []


def test_status_error_carries_the_statement_endpoint_id(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on a statement method still maps to ``FmpStatusError``."""
    fixture_server.route("/income-statement-bulk", {"error": "denied"}, status=402)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.bulk.income_statements(2026, "Q1")
    error = raised.value
    assert error.endpoint == "income-statement-bulk"
    assert error.status == 402
    assert error.body == '{"error": "denied"}'
