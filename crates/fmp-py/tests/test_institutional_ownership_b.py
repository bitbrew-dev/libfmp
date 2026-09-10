"""Runtime contract of ``client.institutional_ownership``: holder and market summaries, negatives.

The expected targets are the ones the Rust
``institutional_holder_summaries_endpoints.rs`` and
``institutional_position_industry_summaries_endpoints.rs`` tests pin. The
negatives cover one invalid value per argument-kind family the domain uses:
``ticker``, ``cik``, ``year``, ``quarter``, ``page``, and ``limit``. The
failure routes prove the status and decode error mapping names the
``institutional-ownership/...`` endpoint id.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.institutional_ownership import (
    HolderIndustryBreakdown,
    HolderPerformanceSummary,
    InstitutionalIndustrySummary,
    InstitutionalPositionSummary,
)

BERKSHIRE_CIK = "0001067983"
APPLE_CIK = "0000320193"
YEAR_2023 = 2023
Q3 = 3
SPACED_SYMBOL = "BRK.B / Class A"
SPACED_SYMBOL_ENCODED = "BRK.B+%2F+Class+A"


def test_holder_performance_summary_with_page(client: Any, fixture_server: FixtureServer) -> None:
    """``holder_performance_summary`` encodes ``cik`` then ``page`` and decodes the summary row."""
    fixture_server.route(
        "/institutional-ownership/holder-performance-summary", load_fixture("holder_performance_summary.json")
    )
    rows = client.institutional_ownership.holder_performance_summary(BERKSHIRE_CIK, page=0)

    assert (
        fixture_server.requests[0].target
        == f"/institutional-ownership/holder-performance-summary?cik={BERKSHIRE_CIK}&page=0"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, HolderPerformanceSummary)
    assert row.cik == BERKSHIRE_CIK
    assert row.investor_name == "BERKSHIRE HATHAWAY INC"
    assert row.date == datetime.date(2026, 3, 31)
    assert row.portfolio_size == 29
    assert row.market_value == 263_095_703_570
    assert row.change_in_performance == -14_398_745_159
    assert row.performance_1_year == 28_972_527_543
    assert row.performance_since_inception_percentage == pytest.approx(203.9112)
    assert row.performance_5_year_relative_to_sp500_percentage == pytest.approx(-1.1428)
    assert row.turnover == pytest.approx(0.6552)


def test_holder_performance_summary_without_page_sends_only_the_cik(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``holder_performance_summary`` with no ``page`` sends only the CIK and rejects a positional page."""
    fixture_server.route(
        "/institutional-ownership/holder-performance-summary", load_fixture("holder_performance_summary.json")
    )
    rows = client.institutional_ownership.holder_performance_summary(BERKSHIRE_CIK)

    assert fixture_server.requests[0].target == f"/institutional-ownership/holder-performance-summary?cik={BERKSHIRE_CIK}"
    assert len(rows) == 1
    with pytest.raises(TypeError):
        client.institutional_ownership.holder_performance_summary(BERKSHIRE_CIK, 0)


def test_holder_industry_breakdown_encodes_cik_year_then_quarter(client: Any, fixture_server: FixtureServer) -> None:
    """``holder_industry_breakdown`` sends the CIK, year, and quarter in order and decodes the row."""
    fixture_server.route(
        "/institutional-ownership/holder-industry-breakdown", load_fixture("holder_industry_breakdown.json")
    )
    rows = client.institutional_ownership.holder_industry_breakdown(BERKSHIRE_CIK, YEAR_2023, Q3)

    assert (
        fixture_server.requests[0].target
        == f"/institutional-ownership/holder-industry-breakdown?cik={BERKSHIRE_CIK}&year=2023&quarter=3"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, HolderIndustryBreakdown)
    assert row.cik == BERKSHIRE_CIK
    assert row.industry_title == "ELECTRONIC COMPUTERS"
    assert row.date == datetime.date(2023, 9, 30)
    assert row.change_in_performance == -47_453_494_598
    assert row.weight == pytest.approx(49.7704)
    assert row.performance_percentage == pytest.approx(-178.2938)


def test_positions_summary_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``positions_summary`` form-encodes the ticker before the year and quarter and decodes the row."""
    fixture_server.route(
        "/institutional-ownership/symbol-positions-summary", load_fixture("institutional_positions_summary.json")
    )
    rows = client.institutional_ownership.positions_summary(SPACED_SYMBOL, YEAR_2023, Q3)

    assert (
        fixture_server.requests[0].target
        == f"/institutional-ownership/symbol-positions-summary?symbol={SPACED_SYMBOL_ENCODED}&year=2023&quarter=3"
    )
    assert fixture_server.requests[0].query["symbol"] == [SPACED_SYMBOL]
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, InstitutionalPositionSummary)
    assert row.symbol == "AAPL"
    assert row.cik == APPLE_CIK
    assert row.date == datetime.date(2023, 9, 30)
    assert row.investors_holding == 4_863
    assert row.number_of_13f_shares == 9_139_920_744
    assert row.total_invested == 1_575_774_922_899
    assert row.total_invested_change == -245_052_087_186
    assert row.ownership_percent == pytest.approx(58.5914)
    assert row.put_call_ratio == pytest.approx(1.1111)


def test_positions_summary_with_a_plain_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``positions_summary`` sends a plain ticker verbatim."""
    fixture_server.route(
        "/institutional-ownership/symbol-positions-summary", load_fixture("institutional_positions_summary.json")
    )
    rows = client.institutional_ownership.positions_summary("AAPL", YEAR_2023, Q3)

    assert (
        fixture_server.requests[0].target
        == "/institutional-ownership/symbol-positions-summary?symbol=AAPL&year=2023&quarter=3"
    )
    assert len(rows) == 1


def test_industry_summary_encodes_year_then_quarter(client: Any, fixture_server: FixtureServer) -> None:
    """``industry_summary`` takes only the reporting period and decodes the industry value row."""
    fixture_server.route(
        "/institutional-ownership/industry-summary", load_fixture("institutional_industry_summary.json")
    )
    rows = client.institutional_ownership.industry_summary(YEAR_2023, Q3)

    assert fixture_server.requests[0].target == "/institutional-ownership/industry-summary?year=2023&quarter=3"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, InstitutionalIndustrySummary)
    assert row.industry_title == "ABRASIVE, ASBESTOS & MISC NONMETALLIC MINERAL PRODS"
    assert row.industry_value == 11_088_059_691
    assert row.date == datetime.date(2023, 9, 30)


@pytest.mark.parametrize("quarter", [1, 2, 3, 4])
def test_industry_summary_accepts_every_quarter(client: Any, fixture_server: FixtureServer, quarter: int) -> None:
    """``industry_summary`` maps each Python quarter integer to its numeric wire value."""
    fixture_server.route(
        "/institutional-ownership/industry-summary", load_fixture("institutional_industry_summary.json")
    )
    client.institutional_ownership.industry_summary(YEAR_2023, quarter)

    assert fixture_server.requests[0].query["quarter"] == [str(quarter)]


@pytest.mark.parametrize(
    ("method", "arguments", "keyword", "message"),
    [
        pytest.param(
            "positions_summary",
            ("   ", YEAR_2023, Q3),
            "symbol",
            "symbol: value must not be empty or whitespace-only",
            id="ticker",
        ),
        pytest.param(
            "form_13f_filing_dates",
            ("",),
            "cik",
            "cik: value must not be empty or whitespace-only",
            id="cik",
        ),
        pytest.param(
            "industry_summary",
            (-1, Q3),
            "year",
            "year: must be an integer from 0 through 4294967295",
            id="year",
        ),
        pytest.param(
            "extract",
            (BERKSHIRE_CIK, YEAR_2023, 5),
            "quarter",
            "quarter: quarter must be an integer from 1 through 4",
            id="quarter-high",
        ),
        pytest.param(
            "holder_industry_breakdown",
            (BERKSHIRE_CIK, YEAR_2023, 0),
            "quarter",
            "quarter: quarter must be an integer from 1 through 4",
            id="quarter-zero",
        ),
    ],
)
def test_invalid_positional_values_name_the_argument(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    arguments: tuple[Any, ...],
    keyword: str,
    message: str,
) -> None:
    """Each required-argument kind fails locally with ``FmpValidationError`` prefixed by the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.institutional_ownership, method)(*arguments)
    error = raised.value
    assert str(error).startswith(message)
    assert str(error).startswith(f"{keyword}: ")
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("method", "arguments", "keyword", "value", "message"),
    [
        pytest.param(
            "latest_filings",
            (),
            "page",
            -1,
            "page: must be an integer from 0 through 4294967295",
            id="page",
        ),
        pytest.param(
            "holder_analytics",
            ("AAPL", YEAR_2023, Q3),
            "limit",
            4_294_967_296,
            "limit: must be an integer from 0 through 4294967295",
            id="limit",
        ),
        pytest.param(
            "holder_performance_summary",
            (BERKSHIRE_CIK,),
            "page",
            -1,
            "page: must be an integer from 0 through 4294967295",
            id="page-on-summary",
        ),
    ],
)
def test_invalid_keyword_values_name_the_argument(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    arguments: tuple[Any, ...],
    keyword: str,
    value: Any,
    message: str,
) -> None:
    """Each optional-setter kind fails locally with ``FmpValidationError`` prefixed by the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.institutional_ownership, method)(*arguments, **{keyword: value})
    error = raised.value
    assert str(error).startswith(message)
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_wrong_argument_types_raise_type_error(client: Any, fixture_server: FixtureServer) -> None:
    """A ``str`` year or a ``float`` quarter is a shape error, reported by pyo3 as ``TypeError``."""
    with pytest.raises(TypeError):
        client.institutional_ownership.industry_summary("2023", Q3)
    with pytest.raises(TypeError):
        client.institutional_ownership.industry_summary(YEAR_2023, 3.0)
    with pytest.raises(TypeError):
        client.institutional_ownership.form_13f_filing_dates(1067983)
    assert fixture_server.requests == []


def test_status_error_carries_the_nested_endpoint_id(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on ``holder_analytics`` names the two-segment endpoint id."""
    fixture_server.route("/institutional-ownership/extract-analytics/holder", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.institutional_ownership.holder_analytics("AAPL", YEAR_2023, Q3)
    error = raised.value
    assert error.endpoint == "institutional-ownership/extract-analytics/holder"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_decode_error_on_a_non_json_body(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-JSON 200 body maps to ``FmpDecodeError`` and names the endpoint."""
    fixture_server.route("/institutional-ownership/latest", b"not-json", content_type="text/plain")
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.institutional_ownership.latest_filings()
    assert raised.value.endpoint == "institutional-ownership/latest"
