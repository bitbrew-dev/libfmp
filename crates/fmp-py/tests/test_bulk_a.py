"""Runtime contract of ``client.bulk`` for the snapshot and metrics methods.

Covers the eleven bulk methods whose bodies live in ``fmp.bulk.snapshots``,
``fmp.bulk.metrics``, and (for ``company_profiles``) ``fmp.company``: one test
per method routes the live CSV fixture body as ``text/csv``, calls the method, and asserts
the exact request target plus a few typed fields. Numeric columns arrive as
``str`` verbatim; only the date columns decode to ``datetime.date``. The expected targets are the ones the Rust
``bulk_snapshot_endpoints.rs`` and ``bulk_metrics_endpoints.rs`` tests pin.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import CSV_CONTENT_TYPE, FixtureServer, load_csv_fixture
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

def route_csv(fixture_server: FixtureServer, path: str, fixture: str) -> None:
    """Serve the shared CSV fixture on ``path`` as ``text/csv``, as the bulk routes answer."""
    fixture_server.route(path, load_csv_fixture(fixture), content_type=CSV_CONTENT_TYPE)


def test_bulk_namespace_is_the_generated_type(client: Any) -> None:
    """``client.bulk`` is the generated flat namespace class."""
    assert isinstance(client.bulk, BulkNamespace)


def test_company_profiles_form_encodes_the_part(client: Any, fixture_server: FixtureServer) -> None:
    """``company_profiles`` sends ``part`` verbatim, form-encoding spaces and slashes."""
    route_csv(fixture_server, "/profile-bulk", "bulk_company_profiles.csv")
    rows = client.bulk.company_profiles("segment 0/alpha")

    assert fixture_server.requests[0].target == "/profile-bulk?part=segment+0%2Falpha"
    assert len(rows) == 4
    row = rows[0]
    assert isinstance(row, CompanyProfile)
    assert row.symbol == "WMB"
    assert row.company_name == "The Williams Companies, Inc."
    assert row.price == pytest.approx(67.78)
    assert row.market_cap == 82_906_291_252
    assert row.is_etf is False
    assert (rows[2].symbol, rows[2].change, rows[2].change_percentage) == ("ACCV", None, None)
    assert (rows[3].symbol, rows[3].isin, rows[3].country) == ("CFHAX", None, None)


def test_company_profiles_with_a_numeric_part(client: Any, fixture_server: FixtureServer) -> None:
    """A plain numeric partition is sent as-is without inferring a range."""
    route_csv(fixture_server, "/profile-bulk", "bulk_company_profiles.csv")
    client.bulk.company_profiles("0")

    assert fixture_server.requests[0].target == "/profile-bulk?part=0"


def test_stock_ratings_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``stock_ratings`` maps to ``/rating-bulk`` with no query string."""
    route_csv(fixture_server, "/rating-bulk", "bulk_stock_ratings.csv")
    rows = client.bulk.stock_ratings()

    assert fixture_server.requests[0].target == "/rating-bulk"
    assert len(rows) == 2
    row = rows[0]
    assert isinstance(row, BulkStockRating)
    assert row.symbol == "000001.SZ"
    assert row.date == datetime.date(2026, 9, 30)
    assert row.rating == "B-"
    assert row.discounted_cash_flow_score == "1"
    with pytest.raises(TypeError):
        client.bulk.stock_ratings("0")


def test_dcf_valuations_decodes_the_spaced_stock_price_column(client: Any, fixture_server: FixtureServer) -> None:
    """``dcf_valuations`` maps the provider's ``Stock Price`` column to ``stock_price``."""
    route_csv(fixture_server, "/dcf-bulk", "bulk_dcf_valuations.csv")
    rows = client.bulk.dcf_valuations()

    assert fixture_server.requests[0].target == "/dcf-bulk"
    assert len(rows) == 3
    row = rows[0]
    assert isinstance(row, BulkDcfValuation)
    assert row.symbol == "000006.SZ"
    assert row.date == datetime.date(2026, 9, 29)
    assert row.dcf == "2.525226853334803"
    assert row.stock_price == "7.62"
    assert rows[2].dcf is None
    assert rows[2].stock_price == "1.72"


def test_financial_scores_keeps_string_backed_numbers(client: Any, fixture_server: FixtureServer) -> None:
    """``financial_scores`` maps to ``/scores-bulk`` and keeps the CSV strings."""
    route_csv(fixture_server, "/scores-bulk", "bulk_financial_scores.csv")
    rows = client.bulk.financial_scores()

    assert fixture_server.requests[0].target == "/scores-bulk"
    assert len(rows) == 3
    row = rows[0]
    assert isinstance(row, BulkFinancialScore)
    assert row.symbol == "000001.SZ"
    assert row.reported_currency == "CNY"
    assert row.altman_z_score == "-0.06634014283050256"
    assert row.piotroski_score == "4"
    assert row.market_cap == "219286875637"
    assert rows[2].reported_currency is None
    assert rows[2].altman_z_score is None


def test_price_target_summaries_preserves_the_raw_publishers_cell(client: Any, fixture_server: FixtureServer) -> None:
    """``price_target_summaries`` keeps the quoted ``publishers`` cell verbatim as one string."""
    route_csv(fixture_server, "/price-target-summary-bulk", "bulk_price_target_summaries.csv")
    rows = client.bulk.price_target_summaries()

    assert fixture_server.requests[0].target == "/price-target-summary-bulk"
    assert len(rows) == 2
    row = rows[0]
    assert isinstance(row, BulkPriceTargetSummary)
    assert row.symbol == "A"
    assert row.all_time_count == "53"
    assert row.all_time_avg_price_target == "159.49"
    assert row.publishers == '["StreetInsider","Benzinga","Pulse 2.0"]'


def test_etf_holdings_form_encodes_the_part(client: Any, fixture_server: FixtureServer) -> None:
    """``etf_holdings`` shares the ``part`` shape and decodes ``lastUpdated`` as a date."""
    route_csv(fixture_server, "/etf-holder-bulk", "bulk_etf_holdings.csv")
    rows = client.bulk.etf_holdings("part 1/beta")

    assert fixture_server.requests[0].target == "/etf-holder-bulk?part=part+1%2Fbeta"
    assert len(rows) == 4
    row = rows[0]
    assert isinstance(row, BulkEtfHolding)
    assert row.symbol == " -- "
    assert row.asset == "TDW"
    assert row.isin == "US88642R1095"
    assert row.weight_percentage == "2.63"
    assert row.last_updated == datetime.date(2026, 9, 27)
    assert (rows[2].asset, rows[2].isin, rows[2].cusip) == (None, None, "")


def test_upgrades_downgrades_consensus_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``upgrades_downgrades_consensus`` maps to its bulk route and keeps the counts as text."""
    route_csv(fixture_server, "/upgrades-downgrades-consensus-bulk", "bulk_upgrades_downgrades_consensus.csv")
    rows = client.bulk.upgrades_downgrades_consensus()

    assert fixture_server.requests[0].target == "/upgrades-downgrades-consensus-bulk"
    assert len(rows) == 2
    row = rows[0]
    assert isinstance(row, BulkUpgradesDowngradesConsensus)
    assert row.symbol == "000550.SZ"
    assert row.consensus == "Buy"
    assert row.buy == "14"
    assert row.strong_buy == "1"


def test_key_metrics_ttm_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``key_metrics_ttm`` maps to ``/key-metrics-ttm-bulk`` with no query string."""
    route_csv(fixture_server, "/key-metrics-ttm-bulk", "bulk_key_metrics_ttm.csv")
    rows = client.bulk.key_metrics_ttm()

    assert fixture_server.requests[0].target == "/key-metrics-ttm-bulk"
    assert len(rows) == 3
    row = rows[0]
    assert isinstance(row, BulkKeyMetricsTtm)
    assert row.symbol == "000001.SZ"
    assert row.market_cap == "224526473551"
    assert row.ev_to_ebitda_ttm == "29.23198788110799"
    assert row.earnings_yield_ttm == "0.19355846887948813"
    assert rows[2].enterprise_value_ttm is None


def test_financial_ratios_ttm_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``financial_ratios_ttm`` maps to ``/ratios-ttm-bulk`` with no query string."""
    route_csv(fixture_server, "/ratios-ttm-bulk", "bulk_financial_ratios_ttm.csv")
    rows = client.bulk.financial_ratios_ttm()

    assert fixture_server.requests[0].target == "/ratios-ttm-bulk"
    assert len(rows) == 3
    row = rows[0]
    assert isinstance(row, BulkFinancialRatiosTtm)
    assert row.symbol == "000001.SZ"
    assert row.asset_turnover_ttm == "0.03463701192046656"
    assert row.book_value_per_share_ttm == "24.12738217279904"
    assert row.net_income_per_ebt_ttm == "0.83539656299258"


def test_stock_peers_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``stock_peers`` maps to ``/peers-bulk`` and keeps the peers cell as one string."""
    route_csv(fixture_server, "/peers-bulk", "bulk_stock_peers.csv")
    rows = client.bulk.stock_peers()

    assert fixture_server.requests[0].target == "/peers-bulk"
    assert len(rows) == 3
    row = rows[0]
    assert isinstance(row, BulkStockPeer)
    assert row.symbol == "000001.SZ"
    assert row.peers == "3698.HK,600000.SS,600015.SS,600016.SS,600036.SS,601166.SS,601658.SS"


def test_earnings_surprises_encodes_the_required_year(client: Any, fixture_server: FixtureServer) -> None:
    """``earnings_surprises`` sends ``year`` as the only query parameter."""
    route_csv(fixture_server, "/earnings-surprises-bulk", "bulk_earnings_surprises.csv")
    rows = client.bulk.earnings_surprises(2026)

    assert fixture_server.requests[0].target == "/earnings-surprises-bulk?year=2026"
    assert len(rows) == 3
    row = rows[0]
    assert isinstance(row, BulkEarningsSurprise)
    assert row.symbol == "AUTO.OL"
    assert row.date == datetime.date(2024, 12, 31)
    assert row.eps_actual == "0.1332"
    assert row.eps_estimated == "0.1581"
    assert row.last_updated == datetime.date(2025, 10, 7)
    assert rows[2].eps_estimated is None


def test_earnings_surprises_accepts_the_year_keyword(client: Any, fixture_server: FixtureServer) -> None:
    """The constructor argument is also reachable by keyword."""
    route_csv(fixture_server, "/earnings-surprises-bulk", "bulk_earnings_surprises.csv")
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
    """A body that is not ``text/csv`` on a bulk method maps to ``FmpDecodeError``."""
    fixture_server.route("/peers-bulk", b"not-csv", content_type="text/plain")
    with pytest.raises(errors.FmpDecodeError):
        client.bulk.stock_peers()
    assert fixture_server.requests[0].target == "/peers-bulk"
