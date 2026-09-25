"""Runtime contract of ``client.bulk`` for the snapshot and metrics methods.

Covers the eleven bulk methods whose bodies live in ``fmp.bulk.snapshots``,
``fmp.bulk.metrics``, and (for ``company_profiles``) ``fmp.company``: one test
per method routes the documented fixture body, calls the method, and asserts
the exact request target plus a few typed fields. Bulk bodies are CSV-backed,
so most numeric columns arrive as ``str``; only the date columns decode to
``datetime.date``. The expected targets are the ones the Rust
``bulk_snapshot_endpoints.rs`` and ``bulk_metrics_endpoints.rs`` tests pin.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.bulk import BulkNamespace
from fmp.bulk.metrics import BulkEarningsSurprise, BulkFinancialRatiosTtm, BulkKeyMetricsTtm, BulkStockPeer
from fmp.bulk.snapshots import (
    BulkDcfValuation,
    BulkEtfHolding,
    BulkFinancialScore,
    BulkPriceTargetSummary,
    BulkStockRating,
    BulkUpgradesDowngradesConsensus,
)
from fmp.company import CompanyProfile

DATE_2025_07_09 = datetime.date(2025, 7, 9)


def test_bulk_namespace_is_the_generated_type(client: Any) -> None:
    """``client.bulk`` is the generated flat namespace class."""
    assert isinstance(client.bulk, BulkNamespace)


def test_company_profiles_form_encodes_the_part(client: Any, fixture_server: FixtureServer) -> None:
    """``company_profiles`` sends ``part`` verbatim, form-encoding spaces and slashes."""
    fixture_server.route("/profile-bulk", load_fixture("bulk_company_profiles.json"))
    rows = client.bulk.company_profiles("segment 0/alpha")

    assert fixture_server.requests[0].target == "/profile-bulk?part=segment+0%2Falpha"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CompanyProfile)
    assert row.symbol == "AAPL"
    assert row.company_name == "Apple Inc."
    assert row.price == pytest.approx(271.36)
    assert row.market_cap == 4_009_711_150_080
    assert row.ipo_date == datetime.date(1980, 12, 12)
    assert row.is_etf is False


def test_company_profiles_with_a_numeric_part(client: Any, fixture_server: FixtureServer) -> None:
    """A plain numeric partition is sent as-is without inferring a range."""
    fixture_server.route("/profile-bulk", load_fixture("bulk_company_profiles.json"))
    client.bulk.company_profiles("0")

    assert fixture_server.requests[0].target == "/profile-bulk?part=0"


def test_stock_ratings_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``stock_ratings`` maps to ``/rating-bulk`` with no query string."""
    fixture_server.route("/rating-bulk", load_fixture("bulk_stock_ratings.json"))
    rows = client.bulk.stock_ratings()

    assert fixture_server.requests[0].target == "/rating-bulk"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, BulkStockRating)
    assert row.symbol == "000001.SZ"
    assert row.date == DATE_2025_07_09
    assert row.rating == "B+"
    assert row.discounted_cash_flow_score == "5"
    with pytest.raises(TypeError):
        client.bulk.stock_ratings("0")


def test_dcf_valuations_decodes_the_spaced_stock_price_column(client: Any, fixture_server: FixtureServer) -> None:
    """``dcf_valuations`` maps the provider's ``Stock Price`` column to ``stock_price``."""
    fixture_server.route("/dcf-bulk", load_fixture("bulk_dcf_valuations.json"))
    rows = client.bulk.dcf_valuations()

    assert fixture_server.requests[0].target == "/dcf-bulk"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, BulkDcfValuation)
    assert row.symbol == "000002.SZ"
    assert row.date == DATE_2025_07_09
    assert row.dcf == "179.6654688379575"
    assert row.stock_price == "6.54"


def test_financial_scores_keeps_string_backed_numbers(client: Any, fixture_server: FixtureServer) -> None:
    """``financial_scores`` maps to ``/scores-bulk`` and keeps the CSV strings."""
    fixture_server.route("/scores-bulk", load_fixture("bulk_financial_scores.json"))
    rows = client.bulk.financial_scores()

    assert fixture_server.requests[0].target == "/scores-bulk"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, BulkFinancialScore)
    assert row.symbol == "000001.SZ"
    assert row.reported_currency == "CNY"
    assert row.altman_z_score == "0.29153682196643543"
    assert row.piotroski_score == "5"
    assert row.market_cap == "236751980000"


def test_price_target_summaries_preserves_the_raw_publishers_cell(client: Any, fixture_server: FixtureServer) -> None:
    """``price_target_summaries`` keeps the provider's malformed ``publishers`` cell verbatim."""
    fixture_server.route("/price-target-summary-bulk", load_fixture("bulk_price_target_summaries.json"))
    rows = client.bulk.price_target_summaries()

    assert fixture_server.requests[0].target == "/price-target-summary-bulk"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, BulkPriceTargetSummary)
    assert row.symbol == "A"
    assert row.all_time_count == "18"
    assert row.all_time_avg_price_target == "146.61"
    assert row.publishers == '[""TheFly"'


def test_etf_holdings_form_encodes_the_part(client: Any, fixture_server: FixtureServer) -> None:
    """``etf_holdings`` shares the ``part`` shape and keeps the raw ``lastUpdated"`` column."""
    fixture_server.route("/etf-holder-bulk", load_fixture("bulk_etf_holdings.json"))
    rows = client.bulk.etf_holdings("part 1/beta")

    assert fixture_server.requests[0].target == "/etf-holder-bulk?part=part+1%2Fbeta"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, BulkEtfHolding)
    assert row.symbol == "EXCH.AS"
    assert row.asset == "009150.KS"
    assert row.isin == "KR7009150004"
    assert row.weight_percentage == "0.09611"
    assert row.last_updated_raw == '2024-09-06"'


def test_upgrades_downgrades_consensus_accepts_an_empty_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``upgrades_downgrades_consensus`` decodes the documented empty-symbol row."""
    fixture_server.route(
        "/upgrades-downgrades-consensus-bulk", load_fixture("bulk_upgrades_downgrades_consensus.json")
    )
    rows = client.bulk.upgrades_downgrades_consensus()

    assert fixture_server.requests[0].target == "/upgrades-downgrades-consensus-bulk"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, BulkUpgradesDowngradesConsensus)
    assert row.symbol == ""
    assert row.consensus == "Buy"
    assert row.buy == "1"
    assert row.strong_buy == "0"


def test_key_metrics_ttm_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``key_metrics_ttm`` maps to ``/key-metrics-ttm-bulk`` with no query string."""
    fixture_server.route("/key-metrics-ttm-bulk", load_fixture("bulk_key_metrics_ttm.json"))
    rows = client.bulk.key_metrics_ttm()

    assert fixture_server.requests[0].target == "/key-metrics-ttm-bulk"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, BulkKeyMetricsTtm)
    assert row.symbol == "000001.SZ"
    assert row.market_cap == "249171756000"
    assert row.ev_to_ebitda_ttm == "-14.656106051669223"
    assert row.earnings_yield_ttm == "0.14960077934639543"


def test_financial_ratios_ttm_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``financial_ratios_ttm`` maps to ``/ratios-ttm-bulk`` with no query string."""
    fixture_server.route("/ratios-ttm-bulk", load_fixture("bulk_financial_ratios_ttm.json"))
    rows = client.bulk.financial_ratios_ttm()

    assert fixture_server.requests[0].target == "/ratios-ttm-bulk"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, BulkFinancialRatiosTtm)
    assert row.symbol == "000001.SZ"
    assert row.asset_turnover_ttm == "0.029075827062555015"
    assert row.book_value_per_share_ttm == "22.260885357516656"
    assert row.net_income_per_ebt_ttm == "0.8225101702576465"


def test_stock_peers_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``stock_peers`` maps to ``/peers-bulk`` and keeps the peers cell as one string."""
    fixture_server.route("/peers-bulk", load_fixture("bulk_stock_peers.json"))
    rows = client.bulk.stock_peers()

    assert fixture_server.requests[0].target == "/peers-bulk"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, BulkStockPeer)
    assert row.symbol == "000001.SZ"
    assert row.peers == "600036.SS"


def test_earnings_surprises_encodes_the_required_year(client: Any, fixture_server: FixtureServer) -> None:
    """``earnings_surprises`` sends ``year`` as the only query parameter."""
    fixture_server.route("/earnings-surprises-bulk", load_fixture("bulk_earnings_surprises.json"))
    rows = client.bulk.earnings_surprises(2026)

    assert fixture_server.requests[0].target == "/earnings-surprises-bulk?year=2026"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, BulkEarningsSurprise)
    assert row.symbol == "AMKYF"
    assert row.date == DATE_2025_07_09
    assert row.eps_actual == "0.3631"
    assert row.eps_estimated == "0.3615"
    assert row.last_updated == DATE_2025_07_09


def test_earnings_surprises_accepts_the_year_keyword(client: Any, fixture_server: FixtureServer) -> None:
    """The constructor argument is also reachable by keyword."""
    fixture_server.route("/earnings-surprises-bulk", load_fixture("bulk_earnings_surprises.json"))
    client.bulk.earnings_surprises(year=2024)

    assert fixture_server.requests[0].target == "/earnings-surprises-bulk?year=2024"


def test_status_error_carries_the_bulk_endpoint_id(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on a bulk method maps to ``FmpStatusError`` with the endpoint id."""
    fixture_server.route("/rating-bulk", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.bulk.stock_ratings()
    error = raised.value
    assert error.endpoint == "rating-bulk"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_decode_error_names_the_bulk_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-JSON body on a bulk method maps to ``FmpDecodeError``."""
    fixture_server.route("/peers-bulk", b"not-json", content_type="text/plain")
    with pytest.raises(errors.FmpDecodeError):
        client.bulk.stock_peers()
    assert fixture_server.requests[0].target == "/peers-bulk"
