"""Runtime contract of ``client.statements`` growth, as-reported, and segmentation.

Covers ``growth`` (four methods on ``StatementPeriod``), ``as_reported`` (four
methods on ``RetrievalFrequency`` plus a free-form ``data`` mapping), and
``segmentation`` (retrieval frequency plus the ``structure`` selector). One
test per method; targets come from the Rust ``statements_*_endpoints.rs`` tests.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.statements.as_reported import AsReportedFinancialStatement, StatementsAsReportedNamespace
from fmp.statements.growth import StatementsGrowthNamespace
from fmp.statements.growth.balance import BalanceSheetStatementGrowth
from fmp.statements.growth.cash_flow import CashFlowStatementGrowth
from fmp.statements.growth.combined import FinancialStatementGrowth
from fmp.statements.growth.income import IncomeStatementGrowth
from fmp.statements.segmentation import RevenueSegmentation, StatementsSegmentationNamespace

DATE_2025_09_27 = datetime.date(2025, 9, 27)
DATE_2025_09_26 = datetime.date(2025, 9, 26)


def test_growth_as_reported_and_segmentation_namespaces(client: Any) -> None:
    """The three sub-namespaces are the generated classes."""
    assert isinstance(client.statements.growth, StatementsGrowthNamespace)
    assert isinstance(client.statements.as_reported, StatementsAsReportedNamespace)
    assert isinstance(client.statements.segmentation, StatementsSegmentationNamespace)


def test_income_statement_growth_with_limit_and_quarter(client: Any, fixture_server: FixtureServer) -> None:
    """``growth.income`` encodes ``symbol``, ``limit``, ``period``."""
    fixture_server.route("/income-statement-growth", load_fixture("income_statement_growth.json"))
    rows = client.statements.growth.income("AAPL", limit=5, period="Q1")

    assert fixture_server.requests[0].target == "/income-statement-growth?symbol=AAPL&limit=5&period=Q1"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, IncomeStatementGrowth)
    assert row.date == DATE_2025_09_27
    assert row.fiscal_year == "2025"
    assert row.growth_revenue == 0.0642551178283274
    assert row.growth_net_income == 0.19495177946573355


def test_balance_sheet_statement_growth_with_quarter_frequency(client: Any, fixture_server: FixtureServer) -> None:
    """``growth.balance_sheet`` accepts the ``quarter`` retrieval frequency."""
    fixture_server.route("/balance-sheet-statement-growth", load_fixture("balance_sheet_statement_growth.json"))
    rows = client.statements.growth.balance_sheet("AAPL", limit=7, period="quarter")

    assert fixture_server.requests[0].target == "/balance-sheet-statement-growth?symbol=AAPL&limit=7&period=quarter"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, BalanceSheetStatementGrowth)
    assert row.date == DATE_2025_09_27
    assert row.period == "FY"
    assert row.growth_total_assets == -0.015724149268453065


def test_cash_flow_statement_growth_with_limit_only(client: Any, fixture_server: FixtureServer) -> None:
    """``growth.cash_flow`` omits ``period`` when it is not given."""
    fixture_server.route("/cash-flow-statement-growth", load_fixture("cash_flow_statement_growth.json"))
    rows = client.statements.growth.cash_flow("AAPL", limit=5)

    assert fixture_server.requests[0].target == "/cash-flow-statement-growth?symbol=AAPL&limit=5"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CashFlowStatementGrowth)
    assert row.date == DATE_2025_09_27
    assert row.growth_net_income == 0.19495177946573355
    assert row.growth_cash_at_end_of_period == pytest.approx(0.20008015228934978, rel=1e-15)


def test_financial_statement_growth_maps_to_financial_growth(client: Any, fixture_server: FixtureServer) -> None:
    """``growth.financial`` uses the provider's ``/financial-growth`` path."""
    fixture_server.route("/financial-growth", load_fixture("financial_statement_growth.json"))
    rows = client.statements.growth.financial("AAPL", period="Q2")

    assert fixture_server.requests[0].target == "/financial-growth?symbol=AAPL&period=Q2"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, FinancialStatementGrowth)
    assert row.date == DATE_2025_09_27
    assert row.revenue_growth == 0.0642551178283274
    assert row.asset_growth == -0.015724149268453065


def _assert_as_reported(row: Any, first_key: str, first_value: int) -> None:
    """Shared field checks for the four as-reported bodies."""
    assert isinstance(row, AsReportedFinancialStatement)
    assert row.symbol == "AAPL"
    assert row.date == DATE_2025_09_26
    assert (row.fiscal_year, row.period, row.reported_currency) == (2025, "FY", "USD")
    assert isinstance(row.data, dict)
    assert row.data[first_key] == first_value


def test_income_as_reported_with_annual_frequency(client: Any, fixture_server: FixtureServer) -> None:
    """``as_reported.income`` takes ``annual`` and exposes the raw ``data`` mapping."""
    fixture_server.route("/income-statement-as-reported", load_fixture("income_statement_as_reported.json"))
    rows = client.statements.as_reported.income("AAPL", limit=5, period="annual")

    assert fixture_server.requests[0].target == "/income-statement-as-reported?symbol=AAPL&limit=5&period=annual"
    assert len(rows) == 1
    _assert_as_reported(rows[0], "comprehensiveincomenetoftax", 113_611_000_000)


def test_null_as_reported_currency_decodes_as_none(client: Any, fixture_server: FixtureServer) -> None:
    """Issue #368: a null ``reportedCurrency`` on an as-reported row decodes as ``None``."""
    rows = load_fixture("income_statement_as_reported.json")
    rows[0]["reportedCurrency"] = None
    fixture_server.route("/income-statement-as-reported", rows)

    row = client.statements.as_reported.income("AAPL")[0]
    assert row.reported_currency is None
    assert row.symbol == "AAPL"


def test_balance_sheet_as_reported_with_quarter_frequency(client: Any, fixture_server: FixtureServer) -> None:
    """``as_reported.balance_sheet`` takes ``quarter``."""
    fixture_server.route(
        "/balance-sheet-statement-as-reported", load_fixture("balance_sheet_statement_as_reported.json")
    )
    rows = client.statements.as_reported.balance_sheet("AAPL", limit=7, period="quarter")

    assert (
        fixture_server.requests[0].target == "/balance-sheet-statement-as-reported?symbol=AAPL&limit=7&period=quarter"
    )
    assert len(rows) == 1
    _assert_as_reported(rows[0], "accountspayablecurrent", 69_860_000_000)


def test_cash_flow_as_reported_with_limit_only(client: Any, fixture_server: FixtureServer) -> None:
    """``as_reported.cash_flow`` sends only ``symbol`` and ``limit``."""
    fixture_server.route("/cash-flow-statement-as-reported", load_fixture("cash_flow_statement_as_reported.json"))
    rows = client.statements.as_reported.cash_flow("AAPL", limit=5)

    assert fixture_server.requests[0].target == "/cash-flow-statement-as-reported?symbol=AAPL&limit=5"
    assert len(rows) == 1
    _assert_as_reported(rows[0], "cashcashequivalentsrestrictedcashandrestrictedcashequivalents", 35_934_000_000)


def test_full_as_reported_with_quarterly_alias(client: Any, fixture_server: FixtureServer) -> None:
    """``as_reported.full`` maps the ``quarterly`` alias to the ``quarter`` wire value."""
    fixture_server.route(
        "/financial-statement-full-as-reported", load_fixture("financial_statement_full_as_reported.json")
    )
    rows = client.statements.as_reported.full("AAPL", period="quarterly")

    assert fixture_server.requests[0].target == "/financial-statement-full-as-reported?symbol=AAPL&period=quarter"
    assert len(rows) == 1
    _assert_as_reported(rows[0], "accountspayablecurrent", 69_860_000_000)


def test_revenue_product_segmentation_with_period_and_structure(client: Any, fixture_server: FixtureServer) -> None:
    """``segmentation.revenue_product`` encodes ``symbol``, ``period``, ``structure``."""
    fixture_server.route("/revenue-product-segmentation", load_fixture("revenue_product_segmentation.json"))
    rows = client.statements.segmentation.revenue_product("AAPL", period="annual", structure="flat")

    assert fixture_server.requests[0].target == "/revenue-product-segmentation?symbol=AAPL&period=annual&structure=flat"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, RevenueSegmentation)
    assert row.date == DATE_2025_09_27
    assert (row.fiscal_year, row.period) == (2025, "FY")
    assert row.data["Mac"] == 33_708_000_000


def test_revenue_geographic_segmentation_without_options(client: Any, fixture_server: FixtureServer) -> None:
    """``segmentation.revenue_geographic`` sends only the symbol by default."""
    fixture_server.route("/revenue-geographic-segmentation", load_fixture("revenue_geographic_segmentation.json"))
    rows = client.statements.segmentation.revenue_geographic("AAPL")

    assert fixture_server.requests[0].target == "/revenue-geographic-segmentation?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, RevenueSegmentation)
    assert row.symbol == "AAPL"
    assert row.date == DATE_2025_09_27
    assert row.data["Americas Segment"] == 178_353_000_000


def test_fiscal_quarter_is_not_a_retrieval_frequency(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """As-reported queries take ``annual``/``quarter`` only; ``Q1`` is rejected locally."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.statements.as_reported.income("AAPL", period="Q1")
    assert str(raised.value) == "period: retrieval frequency must be one of annual, quarter"
    assert fixture_server.requests == []


def test_unknown_segmentation_structure_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """Only the documented ``flat`` structure is accepted."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.statements.segmentation.revenue_product("AAPL", structure="nested")
    assert str(raised.value) == "structure: segmentation structure must be one of flat"
    assert fixture_server.requests == []
