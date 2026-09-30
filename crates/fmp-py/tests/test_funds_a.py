"""Runtime contract of ``client.funds`` for the ETF and latest-holder lookups.

Covers ``etf_holdings``, ``etf_info``, ``etf_country_weightings``,
``etf_asset_exposure``, ``etf_sector_weightings``, and
``latest_disclosure_holders``: one test per method routes the documented
fixture body, calls the method with its single required ticker, and asserts
the exact request target plus a few typed fields (including the
``datetime.date`` and ``datetime.datetime`` ones). The expected targets are
the ones the Rust ``funds_holdings_info_endpoints.rs``,
``funds_allocation_exposure_endpoints.rs``, and
``fund_disclosure_endpoints.rs`` tests pin. The disclosure queries and the
negative cases live in ``test_funds_b.py``.
"""

import datetime
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.funds import (
    EtfAssetExposure,
    EtfCountryWeighting,
    EtfFundHolding,
    EtfFundInfo,
    EtfSectorExposure,
    EtfSectorWeighting,
    FundDisclosureHolder,
    FundsNamespace,
)


def test_funds_namespace_is_the_generated_type(client: Any) -> None:
    """``client.funds`` is the generated flat namespace class."""
    assert isinstance(client.funds, FundsNamespace)


def test_etf_holdings_decodes_the_space_timestamp(client: Any, fixture_server: FixtureServer) -> None:
    """``etf_holdings`` sends only ``symbol`` and decodes the nine-field holding row."""
    fixture_server.route("/etf/holdings", load_fixture("etf_fund_holdings.json"))
    rows = client.funds.etf_holdings("SPY")

    assert fixture_server.requests[0].target == "/etf/holdings?symbol=SPY"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EtfFundHolding)
    assert row.symbol == "SPY"
    assert row.asset == "AAPL"
    assert row.name == "APPLE INC"
    assert row.isin == "US0378331005"
    assert row.security_cusip == "037833100"
    assert row.shares_number == 181_418_073
    assert row.weight_percentage == pytest.approx(7.79997012)
    assert row.market_value == pytest.approx(61_679_458_958.0)
    assert row.updated_at == datetime.datetime(2026, 7, 30, 8, 7, 21)


@pytest.mark.parametrize("wire", [None, ""])
def test_etf_holdings_decodes_null_and_empty_codes_as_none(
    client: Any, fixture_server: FixtureServer, wire: str | None
) -> None:
    """``etf_holdings`` surfaces a null or empty ``asset``, ``isin``, and ``securityCusip`` as ``None``."""
    body = load_fixture("etf_fund_holdings.json")
    body[0].update(asset=wire, isin=wire, securityCusip=wire)
    fixture_server.route("/etf/holdings", body)
    row = client.funds.etf_holdings("SPY")[0]

    assert row.asset is None
    assert row.isin is None
    assert row.security_cusip is None


def test_etf_holdings_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``etf_holdings`` form-encodes spaces and slashes in the ticker exactly as libfmp does."""
    fixture_server.route("/etf/holdings", load_fixture("etf_fund_holdings.json"))
    rows = client.funds.etf_holdings("BRK.B / Class A")

    assert fixture_server.requests[0].target == "/etf/holdings?symbol=BRK.B+%2F+Class+A"
    assert fixture_server.requests[0].query == {"symbol": ["BRK.B / Class A"]}
    assert len(rows) == 1


def test_etf_info_decodes_the_nested_sectors(client: Any, fixture_server: FixtureServer) -> None:
    """``etf_info`` decodes the inception date, the ISO timestamp as text, and the sector list."""
    fixture_server.route("/etf/info", load_fixture("etf_fund_info.json"))
    rows = client.funds.etf_info("SPY")

    assert fixture_server.requests[0].target == "/etf/info?symbol=SPY"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EtfFundInfo)
    assert row.symbol == "SPY"
    assert row.name == "State Street SPDR S&P 500 ETF"
    assert "It also can`t reinvest" in row.description
    assert row.isin == "US78462F1030"
    assert row.asset_class == "Equity"
    assert row.security_cusip == "78462F103"
    assert row.domicile == "US"
    assert row.etf_company == "SPDR"
    assert row.expense_ratio == pytest.approx(0.09)
    assert row.assets_under_management == 777_349_860_000
    assert row.avg_volume == 52_093_933
    assert row.inception_date == datetime.date(1993, 1, 22)
    assert row.nav == pytest.approx(729.27)
    assert row.nav_currency == "USD"
    assert row.holdings_count == 504
    assert row.is_actively_trading is True
    assert row.updated_at == "2026-07-30T16:00:20.049Z"
    assert len(row.sectors_list) == 3
    sector = row.sectors_list[1]
    assert isinstance(sector, EtfSectorExposure)
    assert sector.industry == "Cash & Others"
    assert sector.exposure == pytest.approx(0.30489782336177595)


def test_etf_country_weightings_keep_the_percent_string(client: Any, fixture_server: FixtureServer) -> None:
    """``etf_country_weightings`` preserves the provider's ``97.26%`` string verbatim."""
    fixture_server.route("/etf/country-weightings", load_fixture("etf_country_weightings.json"))
    rows = client.funds.etf_country_weightings("000089.SZ")

    assert fixture_server.requests[0].target == "/etf/country-weightings?symbol=000089.SZ"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EtfCountryWeighting)
    assert row.country == "United States"
    assert row.weight_percentage == "97.26%"


def test_etf_asset_exposure_decodes_numeric_weights(client: Any, fixture_server: FixtureServer) -> None:
    """``etf_asset_exposure`` takes the asset ticker and decodes the numeric weight and value."""
    fixture_server.route("/etf/asset-exposure", load_fixture("etf_asset_exposure.json"))
    rows = client.funds.etf_asset_exposure("AAPL")

    assert fixture_server.requests[0].target == "/etf/asset-exposure?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EtfAssetExposure)
    assert row.symbol == "ZWT-T.TO"
    assert row.asset == "AAPL"
    assert row.shares_number == 42_372
    assert row.weight_percentage == pytest.approx(10.1)
    assert row.market_value == pytest.approx(20_141_231.66)


def test_etf_sector_weightings_decode_the_sector_row(client: Any, fixture_server: FixtureServer) -> None:
    """``etf_sector_weightings`` sends only ``symbol`` and decodes the numeric sector weight."""
    fixture_server.route("/etf/sector-weightings", load_fixture("etf_sector_weightings.json"))
    rows = client.funds.etf_sector_weightings("ZWT-T.TO")

    assert fixture_server.requests[0].target == "/etf/sector-weightings?symbol=ZWT-T.TO"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EtfSectorWeighting)
    assert row.symbol == "SPY"
    assert row.sector == "Basic Materials"
    assert row.weight_percentage == pytest.approx(1.6916311902850854)


def test_latest_fund_disclosure_holders_keep_signed_change(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_disclosure_holders`` preserves the leading-zero CIK and the negative change."""
    fixture_server.route("/funds/disclosure-holders-latest", load_fixture("latest_fund_disclosure_holders.json"))
    rows = client.funds.latest_disclosure_holders("AAPL")

    assert fixture_server.requests[0].target == "/funds/disclosure-holders-latest?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, FundDisclosureHolder)
    assert row.cik == "0000866256"
    assert row.holder == "PARNASSUS INCOME FUNDS"
    assert row.security_cusip == "037833100"
    assert row.shares == 3_638_451
    assert row.date_reported == datetime.date(2026, 6, 30)
    assert row.change == -316_881
    assert row.weight_percent == pytest.approx(4.06607721)


def test_symbol_only_methods_reject_keywords_and_missing_symbol(client: Any) -> None:
    """The six ticker-only methods take exactly one positional ``symbol``."""
    with pytest.raises(TypeError):
        client.funds.etf_info()
    with pytest.raises(TypeError):
        client.funds.etf_country_weightings("SPY", "extra")
    with pytest.raises(TypeError):
        client.funds.latest_disclosure_holders("AAPL", limit=1)


def test_ticker_methods_decode_an_empty_array(client: Any, fixture_server: FixtureServer) -> None:
    """A documented empty body decodes to an empty list on every ticker-only method."""
    for path in [
        "/etf/holdings",
        "/etf/info",
        "/etf/country-weightings",
        "/etf/asset-exposure",
        "/etf/sector-weightings",
        "/funds/disclosure-holders-latest",
    ]:
        fixture_server.route(path, [])

    assert client.funds.etf_holdings("SPY") == []
    assert client.funds.etf_info("SPY") == []
    assert client.funds.etf_country_weightings("SPY") == []
    assert client.funds.etf_asset_exposure("AAPL") == []
    assert client.funds.etf_sector_weightings("SPY") == []
    assert client.funds.latest_disclosure_holders("AAPL") == []
    assert [request.target for request in fixture_server.requests] == [
        "/etf/holdings?symbol=SPY",
        "/etf/info?symbol=SPY",
        "/etf/country-weightings?symbol=SPY",
        "/etf/asset-exposure?symbol=AAPL",
        "/etf/sector-weightings?symbol=SPY",
        "/funds/disclosure-holders-latest?symbol=AAPL",
    ]
