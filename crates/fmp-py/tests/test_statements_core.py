"""Runtime contract of ``client.statements`` for the five core sub-namespaces.

Covers ``income``, ``balance``, ``cash_flow``, ``metrics``, and ``ratios``:
one test per method routes the documented fixture body, calls the method with
one argument shape, and asserts the exact request target plus a few typed
fields (including the ``datetime.date`` ones). The expected targets are the
ones the Rust ``statements_*_endpoints.rs`` tests pin.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.statements import StatementsNamespace
from fmp.statements.balance import BalanceSheetStatement, BalanceSheetStatementTtm, StatementsBalanceNamespace
from fmp.statements.cash_flow import CashFlowStatement, StatementsCashFlowNamespace
from fmp.statements.income import IncomeStatement, StatementsIncomeNamespace
from fmp.statements.metrics import KeyMetrics, KeyMetricsTtm, StatementsMetricsNamespace
from fmp.statements.ratios import FinancialRatios, FinancialRatiosTtm, StatementsRatiosNamespace

DATE_2025_09_27 = datetime.date(2025, 9, 27)
DATE_2026_03_28 = datetime.date(2026, 3, 28)


def test_statements_namespaces_are_the_generated_types(client: Any) -> None:
    """``client.statements`` and each sub-namespace are the generated classes."""
    assert isinstance(client.statements, StatementsNamespace)
    assert isinstance(client.statements.income, StatementsIncomeNamespace)
    assert isinstance(client.statements.balance, StatementsBalanceNamespace)
    assert isinstance(client.statements.cash_flow, StatementsCashFlowNamespace)
    assert isinstance(client.statements.metrics, StatementsMetricsNamespace)
    assert isinstance(client.statements.ratios, StatementsRatiosNamespace)


def test_income_statement_with_period_and_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``income.statement`` encodes ``symbol``, ``limit``, ``period`` in that order."""
    fixture_server.route("/income-statement", load_fixture("income_statement.json"))
    rows = client.statements.income.statement("AAPL", period="annual", limit=5)

    assert fixture_server.requests[0].target == "/income-statement?symbol=AAPL&limit=5&period=annual"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, IncomeStatement)
    assert row.symbol == "AAPL"
    assert row.date == DATE_2025_09_27
    assert row.filing_date == datetime.date(2025, 10, 31)
    assert row.revenue == 416_161_000_000
    assert row.net_income == 112_010_000_000
    assert (row.fiscal_year, row.period) == ("2025", "FY")


def test_income_statement_ttm_with_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``income.statement_ttm`` has no period selector; only ``limit`` is optional."""
    fixture_server.route("/income-statement-ttm", load_fixture("income_statement_ttm.json"))
    rows = client.statements.income.statement_ttm("AAPL", limit=7)

    assert fixture_server.requests[0].target == "/income-statement-ttm?symbol=AAPL&limit=7"
    assert len(rows) == 1
    assert isinstance(rows[0], IncomeStatement)
    assert rows[0].date == DATE_2026_03_28
    assert rows[0].revenue == 451_442_000_000
    assert rows[0].period == "Q2"


def test_balance_sheet_statement_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``balance.statement`` form-encodes the ticker and a fiscal-quarter period."""
    fixture_server.route("/balance-sheet-statement", load_fixture("balance_sheet_statement.json"))
    rows = client.statements.balance.statement("BRK.B / Class A", limit=5, period="Q1")

    assert fixture_server.requests[0].target == "/balance-sheet-statement?symbol=BRK.B+%2F+Class+A&limit=5&period=Q1"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, BalanceSheetStatement)
    assert row.date == DATE_2025_09_27
    assert row.total_assets == 359_241_000_000
    assert row.total_debt == 112_377_000_000
    assert row.cash_and_cash_equivalents == 35_934_000_000


def test_balance_sheet_statement_ttm_without_options(client: Any, fixture_server: FixtureServer) -> None:
    """``balance.statement_ttm`` sends only the symbol when no limit is given."""
    fixture_server.route("/balance-sheet-statement-ttm", load_fixture("balance_sheet_statement_ttm.json"))
    rows = client.statements.balance.statement_ttm("AAPL")

    assert fixture_server.requests[0].target == "/balance-sheet-statement-ttm?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, BalanceSheetStatementTtm)
    assert row.date == DATE_2026_03_28
    assert row.total_assets == 371_082_000_000
    assert row.cash_and_cash_equivalents == 36_328_000_000


def test_cash_flow_statement_with_full_year_period(client: Any, fixture_server: FixtureServer) -> None:
    """``cash_flow.statement`` accepts the ``FY`` fiscal period without a limit."""
    fixture_server.route("/cash-flow-statement", load_fixture("cash_flow_statement.json"))
    rows = client.statements.cash_flow.statement("AAPL", period="FY")

    assert fixture_server.requests[0].target == "/cash-flow-statement?symbol=AAPL&period=FY"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CashFlowStatement)
    assert row.date == DATE_2025_09_27
    assert row.net_income == 112_010_000_000
    assert row.free_cash_flow == 98_767_000_000
    assert row.operating_cash_flow == 111_482_000_000


def test_cash_flow_statement_ttm_with_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``cash_flow.statement_ttm`` decodes the TTM body into ``CashFlowStatement``."""
    fixture_server.route("/cash-flow-statement-ttm", load_fixture("cash_flow_statement_ttm.json"))
    rows = client.statements.cash_flow.statement_ttm("AAPL", limit=7)

    assert fixture_server.requests[0].target == "/cash-flow-statement-ttm?symbol=AAPL&limit=7"
    assert len(rows) == 1
    assert isinstance(rows[0], CashFlowStatement)
    assert rows[0].date == DATE_2026_03_28
    assert rows[0].free_cash_flow == 129_174_000_000
    assert rows[0].net_income == 122_575_000_000


def test_key_metrics_with_fiscal_quarter(client: Any, fixture_server: FixtureServer) -> None:
    """``metrics.key_metrics`` accepts a ``Q3`` fiscal quarter as the period."""
    fixture_server.route("/key-metrics", load_fixture("key_metrics.json"))
    rows = client.statements.metrics.key_metrics("AAPL", limit=5, period="Q3")

    assert fixture_server.requests[0].target == "/key-metrics?symbol=AAPL&limit=5&period=Q3"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, KeyMetrics)
    assert row.date == DATE_2025_09_27
    assert row.market_cap == 3_818_743_810_000
    assert row.enterprise_value == 3_895_186_810_000
    assert row.current_ratio == 0.8932929222186667


def test_key_metrics_ttm_takes_only_a_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``metrics.key_metrics_ttm`` has neither limit nor period."""
    fixture_server.route("/key-metrics-ttm", load_fixture("key_metrics_ttm.json"))
    rows = client.statements.metrics.key_metrics_ttm("AAPL")

    assert fixture_server.requests[0].target == "/key-metrics-ttm?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, KeyMetricsTtm)
    assert row.symbol == "AAPL"
    assert row.market_cap == 4_874_072_686_740
    assert row.current_ratio_ttm == 1.07035746912159
    with pytest.raises(TypeError):
        client.statements.metrics.key_metrics_ttm("AAPL", limit=5)


def test_financial_ratios_with_limit_and_quarter(client: Any, fixture_server: FixtureServer) -> None:
    """``ratios.financial_ratios`` maps to ``/ratios`` with ``Q4`` as the period."""
    fixture_server.route("/ratios", load_fixture("financial_ratios.json"))
    rows = client.statements.ratios.financial_ratios("AAPL", limit=7, period="Q4")

    assert fixture_server.requests[0].target == "/ratios?symbol=AAPL&limit=7&period=Q4"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, FinancialRatios)
    assert row.date == DATE_2025_09_27
    assert row.current_ratio == 0.8932929222186667
    assert row.asset_turnover == 1.1584451663368045


def test_financial_ratios_ttm_takes_only_a_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``ratios.financial_ratios_ttm`` maps to ``/ratios-ttm`` with the symbol alone."""
    fixture_server.route("/ratios-ttm", load_fixture("financial_ratios_ttm.json"))
    rows = client.statements.ratios.financial_ratios_ttm("AAPL")

    assert fixture_server.requests[0].target == "/ratios-ttm?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, FinancialRatiosTtm)
    assert row.symbol == "AAPL"
    assert row.current_ratio_ttm == 1.07035746912159
    assert row.asset_turnover_ttm == 1.2165559094755336


@pytest.mark.parametrize("period", ["monthly", "Q5", ""])
def test_invalid_statement_period_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, period: str
) -> None:
    """A value outside the seven-spelling statement period fails before any request."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.statements.income.statement("AAPL", period=period)
    error = raised.value
    assert str(error) == "period: statement period must be one of Q1, Q2, Q3, Q4, FY, annual, quarter"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_negative_limit_names_the_argument(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A negative ``limit`` is rejected locally with the keyword as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.statements.income.statement("AAPL", limit=-1)
    assert str(raised.value).startswith("limit: ")
    assert fixture_server.requests == []


def test_status_error_carries_the_statement_endpoint_id(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on a nested method still maps to ``FmpStatusError``."""
    fixture_server.route("/income-statement", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.statements.income.statement("AAPL")
    error = raised.value
    assert error.endpoint == "income-statement"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_income_statement_decodes_fractional_share_quantities(client: Any, fixture_server: FixtureServer) -> None:
    """Issue #340: fractional and integral-float weighted share counts decode as ``float``."""
    fixture_server.route("/income-statement", load_fixture("income_statement_fractional_synthetic.json"))
    rows = client.statements.income.statement("AAPL")

    assert len(rows) == 1
    assert isinstance(rows[0].weighted_average_shs_out, float)
    assert rows[0].weighted_average_shs_out == 14_948_500_000.5
    assert rows[0].weighted_average_shs_out_dil == 15_004_697_000.0
