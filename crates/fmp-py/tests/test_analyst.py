"""Runtime contract of ``client.analyst``: estimates, ratings, price targets, stock grades.

One test per method routes the documented fixture body, calls the method with
one argument shape, and asserts the exact request target plus a few typed
fields (including the ``datetime.date`` ones). The expected targets are the
ones the Rust ``analyst_estimates_ratings_endpoints.rs``,
``analyst_price_targets_endpoints.rs``, and ``analyst_stock_grades_endpoints.rs``
tests pin. The negative cases and the error mapping close the file.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.analyst import (
    AnalystNamespace,
    FinancialEstimate,
    HistoricalRating,
    HistoricalStockGrade,
    PriceTargetConsensus,
    PriceTargetSummary,
    RatingSnapshot,
    StockGrade,
    StockGradesSummary,
)

SPACED_SYMBOL = "BRK.B / Class A"
SPACED_SYMBOL_ENCODED = "BRK.B+%2F+Class+A"
U32_MAX = 4_294_967_295
PUBLISHERS = (
    '["StreetInsider","TheFly","Benzinga","Pulse 2.0","TipRanks Contributor",'
    '"MarketWatch","Investing","Barrons","Investor\'s Business Daily"]'
)


def test_analyst_namespace_is_the_generated_type(client: Any) -> None:
    """``client.analyst`` is the generated flat namespace class."""
    assert isinstance(client.analyst, AnalystNamespace)


def test_financial_estimates_annual_without_options(client: Any, fixture_server: FixtureServer) -> None:
    """``financial_estimates`` encodes ``symbol`` then ``period`` and decodes the signed amounts."""
    fixture_server.route("/analyst-estimates", load_fixture("financial_estimates.json"))
    rows = client.analyst.financial_estimates("AAPL", "annual")

    assert fixture_server.requests[0].target == "/analyst-estimates?symbol=AAPL&period=annual"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, FinancialEstimate)
    assert row.symbol == "AAPL"
    assert row.date == datetime.date(2030, 9, 27)
    assert row.revenue_high == 735_022_980_353
    assert row.net_income_low == 191_547_261_069
    assert row.eps_avg == pytest.approx(13.565)
    assert row.num_analysts_revenue == 16
    assert row.num_analysts_eps == 7


def test_financial_estimates_quarter_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """Encoding order is ``symbol``, ``period``, ``page``, ``limit``; ``0`` and ``u32::MAX`` are sent verbatim."""
    fixture_server.route("/analyst-estimates", load_fixture("financial_estimates.json"))
    rows = client.analyst.financial_estimates(SPACED_SYMBOL, "quarter", page=0, limit=U32_MAX)

    assert (
        fixture_server.requests[0].target
        == f"/analyst-estimates?symbol={SPACED_SYMBOL_ENCODED}&period=quarter&page=0&limit={U32_MAX}"
    )
    assert len(rows) == 1
    assert isinstance(rows[0], FinancialEstimate)
    assert rows[0].date == datetime.date(2030, 9, 27)
    assert rows[0].eps_high == pytest.approx(15.01999)


def test_financial_estimates_accepts_period_as_keyword(client: Any, fixture_server: FixtureServer) -> None:
    """``period`` may be passed by keyword; the wire spelling stays ``quarter``."""
    fixture_server.route("/analyst-estimates", load_fixture("financial_estimates.json"))
    rows = client.analyst.financial_estimates("AAPL", period="quarter")

    assert fixture_server.requests[0].target == "/analyst-estimates?symbol=AAPL&period=quarter"
    assert len(rows) == 1


def test_ratings_snapshot_takes_one_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``ratings_snapshot`` maps to ``/ratings-snapshot`` with the symbol alone."""
    fixture_server.route("/ratings-snapshot", load_fixture("ratings_snapshot.json"))
    rows = client.analyst.ratings_snapshot("AAPL")

    assert fixture_server.requests[0].target == "/ratings-snapshot?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, RatingSnapshot)
    assert row.symbol == "AAPL"
    assert row.rating == "B"
    assert row.overall_score == 3
    assert row.return_on_equity_score == 5
    assert row.debt_to_equity_score == 1


def test_historical_ratings_without_options(client: Any, fixture_server: FixtureServer) -> None:
    """``historical_ratings`` maps to ``/ratings-historical`` and decodes the dated row."""
    fixture_server.route("/ratings-historical", load_fixture("historical_ratings.json"))
    rows = client.analyst.historical_ratings("AAPL")

    assert fixture_server.requests[0].target == "/ratings-historical?symbol=AAPL"
    assert "limit=" not in fixture_server.requests[0].raw_query
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, HistoricalRating)
    assert row.date == datetime.date(2026, 7, 30)
    assert row.rating == "B"
    assert row.price_to_book_score == 1


@pytest.mark.parametrize("limit", [0, U32_MAX])
def test_historical_ratings_with_limit(client: Any, fixture_server: FixtureServer, limit: int) -> None:
    """``historical_ratings`` encodes ``symbol`` then ``limit`` across the full ``u32`` domain."""
    fixture_server.route("/ratings-historical", load_fixture("historical_ratings.json"))
    rows = client.analyst.historical_ratings("AAPL", limit=limit)

    assert fixture_server.requests[0].target == f"/ratings-historical?symbol=AAPL&limit={limit}"
    assert len(rows) == 1
    assert rows[0].overall_score == 3


def test_price_target_summary_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``price_target_summary`` form-encodes the ticker and keeps ``publishers`` as the raw string."""
    fixture_server.route("/price-target-summary", load_fixture("price_target_summary.json"))
    rows = client.analyst.price_target_summary(SPACED_SYMBOL)

    assert fixture_server.requests[0].target == f"/price-target-summary?symbol={SPACED_SYMBOL_ENCODED}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, PriceTargetSummary)
    assert row.symbol == "AAPL"
    assert row.publishers == PUBLISHERS
    assert row.all_time_count == 254
    assert row.last_month_count == 8
    assert row.last_month_avg_price_target == pytest.approx(333.75)
    assert row.all_time_avg_price_target == pytest.approx(230.51)


def test_price_target_consensus_decodes_numeric_prices(client: Any, fixture_server: FixtureServer) -> None:
    """``price_target_consensus`` maps to ``/price-target-consensus`` and decodes whole numbers as floats."""
    fixture_server.route("/price-target-consensus", load_fixture("price_target_consensus.json"))
    rows = client.analyst.price_target_consensus("AAPL")

    assert fixture_server.requests[0].target == "/price-target-consensus?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, PriceTargetConsensus)
    assert row.symbol == "AAPL"
    assert isinstance(row.target_high, float)
    assert row.target_high == pytest.approx(400.0)
    assert row.target_low == pytest.approx(250.0)
    assert row.target_consensus == pytest.approx(337.67)
    assert row.target_median == pytest.approx(340.0)


def test_stock_grades_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``stock_grades`` maps to ``/grades`` and decodes the grade action row."""
    fixture_server.route("/grades", load_fixture("stock_grades.json"))
    rows = client.analyst.stock_grades(SPACED_SYMBOL)

    assert fixture_server.requests[0].target == f"/grades?symbol={SPACED_SYMBOL_ENCODED}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockGrade)
    assert row.symbol == "AAPL"
    assert row.date == datetime.date(2026, 7, 23)
    assert row.grading_company == "Morgan Stanley"
    assert row.previous_grade == "Overweight"
    assert row.new_grade == "Overweight"
    assert row.action == "maintain"


def test_historical_stock_grades_without_options(client: Any, fixture_server: FixtureServer) -> None:
    """``historical_stock_grades`` maps to ``/grades-historical`` and decodes the bucket counts."""
    fixture_server.route("/grades-historical", load_fixture("historical_stock_grades.json"))
    rows = client.analyst.historical_stock_grades("AAPL")

    assert fixture_server.requests[0].target == "/grades-historical?symbol=AAPL"
    assert "limit=" not in fixture_server.requests[0].raw_query
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, HistoricalStockGrade)
    assert row.date == datetime.date(2026, 7, 1)
    assert row.analyst_ratings_strong_buy == 6
    assert row.analyst_ratings_buy == 23
    assert row.analyst_ratings_strong_sell == 2


@pytest.mark.parametrize("limit", [0, U32_MAX])
def test_historical_stock_grades_with_limit(client: Any, fixture_server: FixtureServer, limit: int) -> None:
    """``historical_stock_grades`` encodes ``symbol`` then ``limit`` across the full ``u32`` domain."""
    fixture_server.route("/grades-historical", load_fixture("historical_stock_grades.json"))
    rows = client.analyst.historical_stock_grades("AAPL", limit=limit)

    assert fixture_server.requests[0].target == f"/grades-historical?symbol=AAPL&limit={limit}"
    assert len(rows) == 1
    assert rows[0].analyst_ratings_hold == 17


def test_stock_grades_summary_takes_one_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``stock_grades_summary`` maps to ``/grades-consensus`` and decodes the consensus buckets."""
    fixture_server.route("/grades-consensus", load_fixture("stock_grades_summary.json"))
    rows = client.analyst.stock_grades_summary("AAPL")

    assert fixture_server.requests[0].target == "/grades-consensus?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockGradesSummary)
    assert row.symbol == "AAPL"
    assert row.strong_buy == 1
    assert row.buy == 70
    assert row.hold == 32
    assert row.sell == 8
    assert row.strong_sell == 0
    assert row.consensus == "Buy"


def test_stock_grades_summary_accepts_an_integral_float_count(client: Any, fixture_server: FixtureServer) -> None:
    """A count sent as an integral float such as ``3.0`` decodes to the ``int`` ``3``."""
    fixture_server.route("/grades-consensus", [{**load_fixture("stock_grades_summary.json")[0], "buy": 3.0}])
    rows = client.analyst.stock_grades_summary("AAPL")

    assert rows[0].buy == 3
    assert type(rows[0].buy) is int


def test_stock_grades_summary_rejects_a_fractional_count(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A fractional count such as ``2.9`` maps to ``FmpDecodeError`` with the endpoint id."""
    fixture_server.route("/grades-consensus", [{**load_fixture("stock_grades_summary.json")[0], "buy": 2.9}])
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.analyst.stock_grades_summary("AAPL")
    assert raised.value.endpoint == "grades-consensus"


def test_empty_array_decodes_to_an_empty_list(client: Any, fixture_server: FixtureServer) -> None:
    """A bare ``[]`` body is a valid, empty result."""
    fixture_server.route("/grades", [])
    assert client.analyst.stock_grades("AAPL") == []


@pytest.mark.parametrize(
    ("method", "args", "kwargs", "message"),
    [
        pytest.param(
            "ratings_snapshot", ("   ",), {}, "symbol: value must not be empty or whitespace-only", id="ticker"
        ),
        pytest.param(
            "financial_estimates",
            ("AAPL", "Q1"),
            {},
            "period: retrieval frequency must be one of annual, quarter",
            id="retrieval-frequency",
        ),
        pytest.param(
            "financial_estimates",
            ("AAPL", ""),
            {},
            "period: retrieval frequency must be one of annual, quarter",
            id="retrieval-frequency-empty",
        ),
        pytest.param(
            "financial_estimates",
            ("AAPL", "annual"),
            {"page": -1},
            "page: must be an integer from 0 through 4294967295",
            id="page",
        ),
        pytest.param("historical_ratings", ("AAPL",), {"limit": -1}, "limit: ", id="limit"),
        pytest.param("historical_stock_grades", ("AAPL",), {"limit": U32_MAX + 1}, "limit: ", id="limit-overflow"),
    ],
)
def test_invalid_values_name_the_argument(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    args: tuple[Any, ...],
    kwargs: dict[str, Any],
    message: str,
) -> None:
    """Each kind family fails locally with ``FmpValidationError`` prefixed by the argument name."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.analyst, method)(*args, **kwargs)
    error = raised.value
    assert str(error).startswith(message)
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_page_and_limit_are_keyword_only(client: Any, fixture_server: FixtureServer) -> None:
    """The optional setters cannot be passed positionally."""
    with pytest.raises(TypeError):
        client.analyst.financial_estimates("AAPL", "annual", 0)
    with pytest.raises(TypeError):
        client.analyst.historical_ratings("AAPL", 10)
    assert fixture_server.requests == []


def test_status_error_carries_the_price_target_endpoint_id(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on ``price_target_consensus`` names the endpoint."""
    fixture_server.route("/price-target-consensus", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.analyst.price_target_consensus("AAPL")
    error = raised.value
    assert error.endpoint == "price-target-consensus"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_decode_error_on_a_non_json_body(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-JSON 200 body maps to ``FmpDecodeError`` and names the endpoint."""
    fixture_server.route("/grades-consensus", b"not-json", content_type="text/plain")
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.analyst.stock_grades_summary("AAPL")
    assert raised.value.endpoint == "grades-consensus"


def test_decode_error_on_a_non_array_object(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A JSON object where a bare array is documented maps to ``FmpDecodeError`` with the endpoint id."""
    fixture_server.route("/analyst-estimates", {})
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.analyst.financial_estimates("AAPL", "annual")
    assert raised.value.endpoint == "analyst-estimates"
