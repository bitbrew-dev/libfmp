"""Runtime contract of ``client.market`` for the snapshot and historical methods.

One test per method routes the documented fixture body, calls the method with
every optional filter, and asserts the exact request target plus a few typed
fields (including the ``datetime.date`` one). A second call per query family
proves each optional filter is omitted independently. The expected targets are
the ones the Rust ``market_snapshot_endpoints.rs`` and
``market_history_endpoints.rs`` tests pin: the wire order is ``date, exchange,
sector|industry`` for snapshots, ``from, exchange, sector, to`` for sector
history, and ``industry, exchange, from, to`` for industry history. The
market-mover methods and the negatives live in ``test_market_b.py``.
"""

import datetime
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.market import IndustryPe, IndustryPerformance, MarketNamespace, SectorPe, SectorPerformance

SNAPSHOT_DATE = datetime.date(2024, 2, 1)
HISTORY_FROM = datetime.date(2024, 2, 1)
HISTORY_TO = datetime.date(2024, 3, 1)
EXCHANGE = "NEW / EXCHANGE"
EXCHANGE_WIRE = "exchange=NEW+%2F+EXCHANGE"
SECTOR = "Future & Energy / Utilities"
SECTOR_WIRE = "sector=Future+%26+Energy+%2F+Utilities"
INDUSTRY = "Research & Consulting / Services"
INDUSTRY_WIRE = "industry=Research+%26+Consulting+%2F+Services"


def test_market_namespace_is_the_generated_type(client: Any) -> None:
    """``client.market`` is the generated flat namespace class."""
    assert isinstance(client.market, MarketNamespace)


def test_sector_performance_snapshot_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``sector_performance_snapshot`` encodes ``date``, ``exchange``, ``sector`` in that order."""
    fixture_server.route("/sector-performance-snapshot", load_fixture("sector_performance_snapshot.json"))
    rows = client.market.sector_performance_snapshot(SNAPSHOT_DATE, exchange=EXCHANGE, sector="Future & Energy")

    assert (
        fixture_server.requests[0].target
        == "/sector-performance-snapshot?date=2024-02-01&exchange=NEW+%2F+EXCHANGE&sector=Future+%26+Energy"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SectorPerformance)
    assert row.date == SNAPSHOT_DATE
    assert row.sector == "Basic Materials"
    assert row.exchange == "NASDAQ"
    assert row.average_change == pytest.approx(-0.31481377464310634)


def test_sector_performance_snapshot_sends_only_the_exchange(client: Any, fixture_server: FixtureServer) -> None:
    """``sector_performance_snapshot`` accepts an ISO string date and omits ``sector``."""
    fixture_server.route("/sector-performance-snapshot", load_fixture("sector_performance_snapshot.json"))
    rows = client.market.sector_performance_snapshot("2024-02-01", exchange="NASDAQ")

    assert fixture_server.requests[0].target == "/sector-performance-snapshot?date=2024-02-01&exchange=NASDAQ"
    assert len(rows) == 1


def test_industry_performance_snapshot_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``industry_performance_snapshot`` encodes ``date``, ``exchange``, ``industry`` in that order."""
    fixture_server.route("/industry-performance-snapshot", load_fixture("industry_performance_snapshot.json"))
    rows = client.market.industry_performance_snapshot(SNAPSHOT_DATE, exchange=EXCHANGE, industry=INDUSTRY)

    assert fixture_server.requests[0].target == f"/industry-performance-snapshot?date=2024-02-01&{EXCHANGE_WIRE}&{INDUSTRY_WIRE}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, IndustryPerformance)
    assert row.date == SNAPSHOT_DATE
    assert row.industry == "Advertising Agencies"
    assert row.exchange == "NASDAQ"
    assert row.average_change == pytest.approx(3.8660194344955996)


def test_industry_performance_snapshot_sends_only_the_industry(client: Any, fixture_server: FixtureServer) -> None:
    """``industry_performance_snapshot`` omits ``exchange`` when only ``industry`` is given."""
    fixture_server.route("/industry-performance-snapshot", load_fixture("industry_performance_snapshot.json"))
    rows = client.market.industry_performance_snapshot(SNAPSHOT_DATE, industry="Biotechnology")

    assert fixture_server.requests[0].target == "/industry-performance-snapshot?date=2024-02-01&industry=Biotechnology"
    assert len(rows) == 1


def test_sector_pe_snapshot_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``sector_pe_snapshot`` maps to ``/sector-pe-snapshot`` and decodes the ``pe`` float."""
    fixture_server.route("/sector-pe-snapshot", load_fixture("sector_pe_snapshot.json"))
    rows = client.market.sector_pe_snapshot(SNAPSHOT_DATE, exchange=EXCHANGE, sector="Future & Energy")

    assert (
        fixture_server.requests[0].target
        == "/sector-pe-snapshot?date=2024-02-01&exchange=NEW+%2F+EXCHANGE&sector=Future+%26+Energy"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SectorPe)
    assert row.date == SNAPSHOT_DATE
    assert row.sector == "Basic Materials"
    assert row.exchange == "NASDAQ"
    assert row.pe == pytest.approx(15.687711758428254)


def test_sector_pe_snapshot_sends_only_the_sector(client: Any, fixture_server: FixtureServer) -> None:
    """``sector_pe_snapshot`` omits ``exchange`` when only ``sector`` is given."""
    fixture_server.route("/sector-pe-snapshot", load_fixture("sector_pe_snapshot.json"))
    rows = client.market.sector_pe_snapshot("2024-02-01", sector="Energy")

    assert fixture_server.requests[0].target == "/sector-pe-snapshot?date=2024-02-01&sector=Energy"
    assert len(rows) == 1


def test_industry_pe_snapshot_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``industry_pe_snapshot`` maps to ``/industry-pe-snapshot`` and decodes the ``pe`` float."""
    fixture_server.route("/industry-pe-snapshot", load_fixture("industry_pe_snapshot.json"))
    rows = client.market.industry_pe_snapshot(SNAPSHOT_DATE, exchange=EXCHANGE, industry=INDUSTRY)

    assert fixture_server.requests[0].target == f"/industry-pe-snapshot?date=2024-02-01&{EXCHANGE_WIRE}&{INDUSTRY_WIRE}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, IndustryPe)
    assert row.date == SNAPSHOT_DATE
    assert row.industry == "Advertising Agencies"
    assert row.exchange == "NASDAQ"
    assert row.pe == pytest.approx(71.09601665201151)


def test_industry_pe_snapshot_without_options_sends_only_the_date(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``industry_pe_snapshot`` sends only ``date`` when neither filter is given."""
    fixture_server.route("/industry-pe-snapshot", load_fixture("industry_pe_snapshot.json"))
    rows = client.market.industry_pe_snapshot(SNAPSHOT_DATE)

    assert fixture_server.requests[0].target == "/industry-pe-snapshot?date=2024-02-01"
    assert len(rows) == 1


def test_historical_sector_performance_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``historical_sector_performance`` encodes ``from``, ``exchange``, ``sector``, ``to`` in that order."""
    fixture_server.route("/historical-sector-performance", load_fixture("historical_sector_performance.json"))
    rows = client.market.historical_sector_performance(SECTOR, from_=HISTORY_FROM, exchange=EXCHANGE, to=HISTORY_TO)

    assert (
        fixture_server.requests[0].target
        == f"/historical-sector-performance?from=2024-02-01&{EXCHANGE_WIRE}&{SECTOR_WIRE}&to=2024-03-01"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SectorPerformance)
    assert row.date == HISTORY_TO
    assert row.sector == "Energy"
    assert row.exchange == "NASDAQ"
    assert row.average_change == pytest.approx(1.3989969286740689)


def test_historical_sector_performance_without_options_sends_only_the_sector(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``historical_sector_performance`` sends only ``sector`` when no filter is given."""
    fixture_server.route("/historical-sector-performance", load_fixture("historical_sector_performance.json"))
    rows = client.market.historical_sector_performance(SECTOR)

    assert fixture_server.requests[0].target == f"/historical-sector-performance?{SECTOR_WIRE}"
    assert len(rows) == 1


def test_historical_sector_performance_omits_from_independently(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``historical_sector_performance`` keeps ``exchange`` and ``to`` when ``from_`` is left out."""
    fixture_server.route("/historical-sector-performance", load_fixture("historical_sector_performance.json"))
    rows = client.market.historical_sector_performance(SECTOR, exchange=EXCHANGE, to="2024-03-01")

    assert fixture_server.requests[0].target == f"/historical-sector-performance?{EXCHANGE_WIRE}&{SECTOR_WIRE}&to=2024-03-01"
    assert len(rows) == 1


def test_historical_industry_performance_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``historical_industry_performance`` encodes ``industry``, ``exchange``, ``from``, ``to`` in that order."""
    fixture_server.route("/historical-industry-performance", load_fixture("historical_industry_performance.json"))
    rows = client.market.historical_industry_performance(INDUSTRY, exchange=EXCHANGE, from_=HISTORY_FROM, to=HISTORY_TO)

    assert (
        fixture_server.requests[0].target
        == f"/historical-industry-performance?{INDUSTRY_WIRE}&{EXCHANGE_WIRE}&from=2024-02-01&to=2024-03-01"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, IndustryPerformance)
    assert row.date == HISTORY_TO
    assert row.industry == "Biotechnology"
    assert row.exchange == "NASDAQ"
    assert row.average_change == pytest.approx(2.6143442556463383)


def test_historical_industry_performance_omits_from_and_to(client: Any, fixture_server: FixtureServer) -> None:
    """``historical_industry_performance`` sends ``industry`` and ``exchange`` only."""
    fixture_server.route("/historical-industry-performance", load_fixture("historical_industry_performance.json"))
    rows = client.market.historical_industry_performance(INDUSTRY, exchange=EXCHANGE)

    assert fixture_server.requests[0].target == f"/historical-industry-performance?{INDUSTRY_WIRE}&{EXCHANGE_WIRE}"
    assert len(rows) == 1


def test_historical_sector_pe_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``historical_sector_pe`` maps to ``/historical-sector-pe`` with the sector wire order."""
    fixture_server.route("/historical-sector-pe", load_fixture("historical_sector_pe.json"))
    rows = client.market.historical_sector_pe(SECTOR, from_="2024-02-01", exchange=EXCHANGE, to="2024-03-01")

    assert (
        fixture_server.requests[0].target
        == f"/historical-sector-pe?from=2024-02-01&{EXCHANGE_WIRE}&{SECTOR_WIRE}&to=2024-03-01"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SectorPe)
    assert row.date == HISTORY_TO
    assert row.sector == "Energy"
    assert row.exchange == "NASDAQ"
    assert row.pe == pytest.approx(5.4165892628211205)


def test_historical_sector_pe_omits_exchange_and_to(client: Any, fixture_server: FixtureServer) -> None:
    """``historical_sector_pe`` sends ``from`` before ``sector`` and nothing else."""
    fixture_server.route("/historical-sector-pe", load_fixture("historical_sector_pe.json"))
    rows = client.market.historical_sector_pe(SECTOR, from_=HISTORY_FROM)

    assert fixture_server.requests[0].target == f"/historical-sector-pe?from=2024-02-01&{SECTOR_WIRE}"
    assert len(rows) == 1


def test_historical_sector_pe_omits_from_and_exchange(client: Any, fixture_server: FixtureServer) -> None:
    """``historical_sector_pe`` sends ``sector`` then ``to`` when only ``to`` is given."""
    fixture_server.route("/historical-sector-pe", load_fixture("historical_sector_pe.json"))
    rows = client.market.historical_sector_pe(SECTOR, to=HISTORY_TO)

    assert fixture_server.requests[0].target == f"/historical-sector-pe?{SECTOR_WIRE}&to=2024-03-01"
    assert len(rows) == 1


def test_historical_industry_pe_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``historical_industry_pe`` maps to ``/historical-industry-pe`` with the industry wire order."""
    fixture_server.route("/historical-industry-pe", load_fixture("historical_industry_pe.json"))
    rows = client.market.historical_industry_pe(INDUSTRY, exchange=EXCHANGE, from_="2024-02-01", to=HISTORY_TO)

    assert (
        fixture_server.requests[0].target
        == f"/historical-industry-pe?{INDUSTRY_WIRE}&{EXCHANGE_WIRE}&from=2024-02-01&to=2024-03-01"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, IndustryPe)
    assert row.date == HISTORY_TO
    assert row.industry == "Biotechnology"
    assert row.exchange == "NASDAQ"
    assert row.pe == pytest.approx(8.129037884885042)


def test_historical_industry_pe_omits_exchange_independently(client: Any, fixture_server: FixtureServer) -> None:
    """``historical_industry_pe`` keeps both dates when ``exchange`` is left out."""
    fixture_server.route("/historical-industry-pe", load_fixture("historical_industry_pe.json"))
    rows = client.market.historical_industry_pe(INDUSTRY, from_=HISTORY_FROM, to=HISTORY_TO)

    assert fixture_server.requests[0].target == f"/historical-industry-pe?{INDUSTRY_WIRE}&from=2024-02-01&to=2024-03-01"
    assert len(rows) == 1


def test_historical_industry_pe_without_options_sends_only_the_industry(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``historical_industry_pe`` sends only ``industry`` when no filter is given."""
    fixture_server.route("/historical-industry-pe", load_fixture("historical_industry_pe.json"))
    rows = client.market.historical_industry_pe(INDUSTRY)

    assert fixture_server.requests[0].target == f"/historical-industry-pe?{INDUSTRY_WIRE}"
    assert len(rows) == 1
