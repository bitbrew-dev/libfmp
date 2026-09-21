"""Runtime contract of ``client.dcf`` for the four valuation methods.

The two plain methods take a symbol; the two custom methods flatten the
eighteen ``DcfAssumptions`` setters into keyword-only ``float`` arguments and
send only the ones that were passed, in the order the Rust query encodes
them. Every expected target is the one the Rust ``dcf_endpoints.rs`` and
``dcf_custom_endpoints.rs`` tests pin, including the full eighteen-assumption
query string. The negatives cover the ``ticker`` and ``finite_decimal`` kinds,
keyword-only enforcement, and the structured status and decode failures.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.dcf import CustomDcfValuation, CustomLeveredDcfValuation, DcfNamespace, DcfValuation

SPACED_SYMBOL = "BRK.B / Class A"
SPACED_SYMBOL_ENCODED = "BRK.B+%2F+Class+A"
CUSTOM_PATH = "/custom-discounted-cash-flow"
CUSTOM_LEVERED_PATH = "/custom-levered-discounted-cash-flow"
CUSTOM_METHODS = [
    pytest.param("custom_discounted_cash_flow", CUSTOM_PATH, id="custom"),
    pytest.param("custom_levered_discounted_cash_flow", CUSTOM_LEVERED_PATH, id="custom-levered"),
]

DOCUMENTED_ASSUMPTIONS: dict[str, float] = {
    "revenue_growth_pct": 0.1094119804597946,
    "ebitda_pct": 0.31273548388,
    "depreciation_and_amortization_pct": 0.0345531631720999,
    "cash_and_short_term_investments_pct": 0.2344222126801843,
    "receivables_pct": 0.1533770531229388,
    "inventories_pct": 0.0155245674227653,
    "payable_pct": 0.1614868903169657,
    "ebit_pct": 0.2781823207138459,
    "capital_expenditure_pct": 0.0306025847141713,
    "operating_cash_flow_pct": 0.2886333485760204,
    "selling_general_and_administrative_expenses_pct": 0.0662854095187211,
    "tax_rate": 0.14919579658453103,
    "long_term_growth_rate": 4.0,
    "cost_of_debt": 3.64,
    "cost_of_equity": 9.51168,
    "market_risk_premium": 4.72,
    "beta": 1.244,
    "risk_free_rate": 3.64,
}
DOCUMENTED_QUERY = (
    f"symbol={SPACED_SYMBOL_ENCODED}&revenueGrowthPct=0.1094119804597946&ebitdaPct=0.31273548388"
    "&depreciationAndAmortizationPct=0.0345531631720999&cashAndShortTermInvestmentsPct=0.2344222126801843"
    "&receivablesPct=0.1533770531229388&inventoriesPct=0.0155245674227653&payablePct=0.1614868903169657"
    "&ebitPct=0.2781823207138459&capitalExpenditurePct=0.0306025847141713&operatingCashFlowPct=0.2886333485760204"
    "&sellingGeneralAndAdministrativeExpensesPct=0.0662854095187211&taxRate=0.14919579658453103"
    "&longTermGrowthRate=4&costOfDebt=3.64&costOfEquity=9.51168&marketRiskPremium=4.72&beta=1.244&riskFreeRate=3.64"
)
WIRE_KEYS: dict[str, str] = {
    "revenue_growth_pct": "revenueGrowthPct",
    "ebitda_pct": "ebitdaPct",
    "depreciation_and_amortization_pct": "depreciationAndAmortizationPct",
    "cash_and_short_term_investments_pct": "cashAndShortTermInvestmentsPct",
    "receivables_pct": "receivablesPct",
    "inventories_pct": "inventoriesPct",
    "payable_pct": "payablePct",
    "ebit_pct": "ebitPct",
    "capital_expenditure_pct": "capitalExpenditurePct",
    "operating_cash_flow_pct": "operatingCashFlowPct",
    "selling_general_and_administrative_expenses_pct": "sellingGeneralAndAdministrativeExpensesPct",
    "tax_rate": "taxRate",
    "long_term_growth_rate": "longTermGrowthRate",
    "cost_of_debt": "costOfDebt",
    "cost_of_equity": "costOfEquity",
    "market_risk_premium": "marketRiskPremium",
    "beta": "beta",
    "risk_free_rate": "riskFreeRate",
}


def test_dcf_namespace_is_the_generated_type(client: Any) -> None:
    """``client.dcf`` is the generated flat namespace class."""
    assert isinstance(client.dcf, DcfNamespace)


def test_discounted_cash_flow_decodes_the_documented_fixture(client: Any, fixture_server: FixtureServer) -> None:
    """``discounted_cash_flow`` maps to ``/discounted-cash-flow`` with a form-encoded symbol and decodes the row."""
    fixture_server.route("/discounted-cash-flow", load_fixture("discounted_cash_flow.json"))
    rows = client.dcf.discounted_cash_flow(SPACED_SYMBOL)

    assert fixture_server.requests[0].target == f"/discounted-cash-flow?symbol={SPACED_SYMBOL_ENCODED}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, DcfValuation)
    assert row.symbol == "AAPL"
    assert row.date == datetime.date(2026, 7, 30)
    assert row.dcf == pytest.approx(147.10881272667325)
    assert row.stock_price == pytest.approx(338.19)


def test_levered_discounted_cash_flow_decodes_the_documented_fixture(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``levered_discounted_cash_flow`` shares the row model and sends only ``symbol``."""
    fixture_server.route("/levered-discounted-cash-flow", load_fixture("levered_discounted_cash_flow.json"))
    rows = client.dcf.levered_discounted_cash_flow("AAPL")

    assert fixture_server.requests[0].target == "/levered-discounted-cash-flow?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, DcfValuation)
    assert isinstance(row.date, datetime.date)
    assert row.dcf == pytest.approx(140.6429495133426)


@pytest.mark.parametrize("method", ["discounted_cash_flow", "levered_discounted_cash_flow"])
def test_plain_methods_take_exactly_one_symbol(client: Any, fixture_server: FixtureServer, method: str) -> None:
    """The plain methods require ``symbol`` and accept no assumption keywords."""
    with pytest.raises(TypeError):
        getattr(client.dcf, method)()
    with pytest.raises(TypeError):
        getattr(client.dcf, method)("AAPL", beta=1.0)
    assert fixture_server.requests == []


def test_custom_discounted_cash_flow_without_assumptions(client: Any, fixture_server: FixtureServer) -> None:
    """With no assumptions the custom route carries only ``symbol`` and decodes the unlevered row."""
    fixture_server.route(CUSTOM_PATH, load_fixture("custom_discounted_cash_flow.json"))
    rows = client.dcf.custom_discounted_cash_flow("AAPL")

    assert fixture_server.requests[0].target == f"{CUSTOM_PATH}?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CustomDcfValuation)
    assert row.year == "2030"
    assert row.symbol == "AAPL"
    assert row.capital_expenditure == -14_907_445_037
    assert row.diluted_shares_outstanding == 15_004_697_000
    assert row.equity_value_per_share == pytest.approx(147.18)
    assert isinstance(row.beta, float)


def test_custom_levered_discounted_cash_flow_without_assumptions(client: Any, fixture_server: FixtureServer) -> None:
    """With no assumptions the levered custom route carries only ``symbol`` and decodes the levered row."""
    fixture_server.route(CUSTOM_LEVERED_PATH, load_fixture("custom_levered_discounted_cash_flow.json"))
    rows = client.dcf.custom_levered_discounted_cash_flow("AAPL")

    assert fixture_server.requests[0].target == f"{CUSTOM_LEVERED_PATH}?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CustomLeveredDcfValuation)
    assert row.year == "2030"
    assert row.operating_cash_flow == 153_867_620_418
    assert row.pv_lfcf == 88_605_139_549
    assert row.equity_value_per_share == pytest.approx(140.71)
    assert row.wacc == pytest.approx(9.42)


@pytest.mark.parametrize(("method", "path"), CUSTOM_METHODS)
def test_custom_methods_encode_every_assumption_in_documented_order(
    client: Any, fixture_server: FixtureServer, method: str, path: str
) -> None:
    """All eighteen assumptions follow ``symbol`` in the Rust encoding order, at their exact magnitudes."""
    fixture_server.route(path, load_fixture(f"{method}.json"))
    rows = getattr(client.dcf, method)(SPACED_SYMBOL, **DOCUMENTED_ASSUMPTIONS)

    assert fixture_server.requests[0].target == f"{path}?{DOCUMENTED_QUERY}"
    assert len(rows) == 1
    assert rows[0].long_term_growth_rate == pytest.approx(4.0)


@pytest.mark.parametrize(("method", "path"), CUSTOM_METHODS)
@pytest.mark.parametrize(("keyword", "wire_key"), list(WIRE_KEYS.items()))
def test_each_assumption_is_sent_alone_under_its_wire_key(
    client: Any, fixture_server: FixtureServer, method: str, path: str, keyword: str, wire_key: str
) -> None:
    """One assumption at a time reaches the wire under its camelCase key, after ``symbol``."""
    fixture_server.route(path, load_fixture(f"{method}.json"))
    rows = getattr(client.dcf, method)("AAPL", **{keyword: 1.25})

    assert fixture_server.requests[0].target == f"{path}?symbol=AAPL&{wire_key}=1.25"
    assert fixture_server.requests[0].query == {"symbol": ["AAPL"], wire_key: ["1.25"]}
    assert len(rows) == 1


def test_assumptions_keep_encoding_order_regardless_of_keyword_order(
    client: Any, fixture_server: FixtureServer
) -> None:
    """Keyword order does not matter: ``beta`` precedes ``riskFreeRate`` on the wire and the rest stay omitted."""
    fixture_server.route(CUSTOM_PATH, load_fixture("custom_discounted_cash_flow.json"))
    client.dcf.custom_discounted_cash_flow("AAPL", risk_free_rate=3.64, beta=1.244)

    assert fixture_server.requests[0].target == f"{CUSTOM_PATH}?symbol=AAPL&beta=1.244&riskFreeRate=3.64"


def test_zero_negative_and_integral_assumptions_are_encoded_exactly(client: Any, fixture_server: FixtureServer) -> None:
    """``0.0`` is sent as ``0``, negatives keep their sign, and a Python ``int`` is accepted for a float."""
    fixture_server.route(CUSTOM_LEVERED_PATH, load_fixture("custom_levered_discounted_cash_flow.json"))
    client.dcf.custom_levered_discounted_cash_flow(
        "AAPL", revenue_growth_pct=0.0, tax_rate=-1.25, long_term_growth_rate=4
    )

    assert (
        fixture_server.requests[0].target
        == f"{CUSTOM_LEVERED_PATH}?symbol=AAPL&revenueGrowthPct=0&taxRate=-1.25&longTermGrowthRate=4"
    )


@pytest.mark.parametrize(("method", "path"), CUSTOM_METHODS)
def test_assumptions_are_keyword_only_and_the_builder_never_leaks(
    client: Any, fixture_server: FixtureServer, method: str, path: str
) -> None:
    """A positional assumption or an ``assumptions`` keyword is a ``TypeError`` before any request."""
    with pytest.raises(TypeError):
        getattr(client.dcf, method)("AAPL", 1.244)
    with pytest.raises(TypeError, match="unexpected keyword argument 'assumptions'"):
        getattr(client.dcf, method)("AAPL", assumptions={"beta": 1.244})
    with pytest.raises(TypeError):
        getattr(client.dcf, method)()
    assert fixture_server.requests == []


@pytest.mark.parametrize(("method", "path"), CUSTOM_METHODS)
@pytest.mark.parametrize(
    ("keyword", "value"),
    [
        ("revenue_growth_pct", float("nan")),
        ("beta", float("inf")),
        ("risk_free_rate", float("-inf")),
    ],
)
def test_non_finite_assumption_names_the_keyword(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    path: str,
    keyword: str,
    value: float,
) -> None:
    """A non-finite assumption is rejected locally with the keyword as the message prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.dcf, method)("AAPL", **{keyword: value})
    error = raised.value
    assert str(error) == f"{keyword}: decimal value must be finite"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_non_numeric_assumption_is_a_type_error(client: Any, fixture_server: FixtureServer) -> None:
    """A string where a float is expected fails at extraction, before validation or any request."""
    with pytest.raises(TypeError):
        client.dcf.custom_discounted_cash_flow("AAPL", beta="1.244")
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    "method",
    [
        "discounted_cash_flow",
        "levered_discounted_cash_flow",
        "custom_discounted_cash_flow",
        "custom_levered_discounted_cash_flow",
    ],
)
@pytest.mark.parametrize(
    ("value", "message"),
    [
        ("", "symbol: value must not be empty or whitespace-only"),
        ("AAPL,MSFT", "symbol: ticker must not contain a comma"),
    ],
)
def test_invalid_symbol_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, value: str, message: str
) -> None:
    """Every method validates ``symbol`` locally with the argument name as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.dcf, method)(value)
    assert str(raised.value) == message
    assert fixture_server.requests == []


def test_status_error_names_the_plain_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status carries the libfmp endpoint id, status, and body."""
    fixture_server.route("/discounted-cash-flow", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.dcf.discounted_cash_flow("AAPL")
    error = raised.value
    assert error.endpoint == "discounted-cash-flow"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_status_error_names_the_custom_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A failing custom route reports the ``custom-discounted-cash-flow`` id and the raw body."""
    fixture_server.route(CUSTOM_PATH, b"not-json", status=500, content_type="text/plain")
    with pytest.raises(errors.FmpStatusError) as raised:
        client.dcf.custom_discounted_cash_flow("AAPL", beta=1.244)
    assert raised.value.endpoint == "custom-discounted-cash-flow"
    assert raised.value.status == 500
    assert raised.value.body == "not-json"


def test_decode_error_names_the_levered_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-array body on the levered route surfaces as a decode error with the endpoint id."""
    fixture_server.route("/levered-discounted-cash-flow", {})
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.dcf.levered_discounted_cash_flow("AAPL")
    assert raised.value.endpoint == "levered-discounted-cash-flow"


def test_decode_error_names_the_custom_levered_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A well-formed but incomplete levered row is a decode error naming the custom levered endpoint."""
    fixture_server.route(CUSTOM_LEVERED_PATH, [{"symbol": "AAPL", "year": "2030"}])
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.dcf.custom_levered_discounted_cash_flow("AAPL")
    assert raised.value.endpoint == "custom-levered-discounted-cash-flow"
