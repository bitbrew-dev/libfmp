"""Runtime contract of ``client.congressional``: the eight chamber trade feeds.

One test per method routes the documented fixture body, calls the method with
one argument shape, and asserts the exact request target plus a few typed
fields (including the two ``datetime.date`` ones). The expected targets are
the ones the Rust ``congressional_trade_endpoints.rs`` test pins. Profiles,
positions, net worth, and the negative cases live in ``test_congressional_b.py``.
"""

import datetime
from typing import Any

from conftest import FixtureServer, load_fixture
from fmp.congressional import CongressionalNamespace, CongressionalTrade

SPACED_SYMBOL = "BRK.B / Class A"
SPACED_SYMBOL_ENCODED = "BRK.B+%2F+Class+A"
SENATE_LINK = "https://efdsearch.senate.gov/search/view/ptr/bccf83ce-dd72-4ab6-8564-b3bbb1d2ee55/"
HOUSE_LINK = "https://disclosures-clerk.house.gov/public_disc/ptr-pdfs/2026/20034963.pdf"


def test_congressional_namespace_is_the_generated_type(client: Any) -> None:
    """``client.congressional`` is the generated flat namespace class."""
    assert isinstance(client.congressional, CongressionalNamespace)


def test_latest_senate_disclosures_with_page_and_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_senate_disclosures`` encodes ``page`` then ``limit`` and decodes both dates."""
    fixture_server.route("/senate-latest", load_fixture("congress_senate_latest.json"))
    rows = client.congressional.latest_senate_disclosures(page=0, limit=250)

    assert fixture_server.requests[0].target == "/senate-latest?page=0&limit=250"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CongressionalTrade)
    assert row.symbol == "CM"
    assert row.member_id == "M001242"
    assert row.disclosure_date == datetime.date(2026, 7, 24)
    assert row.transaction_date == datetime.date(2026, 6, 22)
    assert row.first_name == "Bernie"
    assert row.last_name == "Moreno"
    assert row.asset_type == "Corporate Bond"
    assert row.transaction_type == "Sale"
    assert row.amount == "$1,001 - $15,000"
    assert row.capital_gains_over_200_usd is None
    assert row.link == SENATE_LINK


def test_latest_senate_disclosures_without_options_sends_the_bare_path(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``latest_senate_disclosures`` with nothing set requests the bare path."""
    fixture_server.route("/senate-latest", load_fixture("congress_senate_latest.json"))
    rows = client.congressional.latest_senate_disclosures()

    assert fixture_server.requests[0].target == "/senate-latest"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1


def test_latest_house_disclosures_emits_a_zero_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_house_disclosures`` sends ``limit=0`` rather than dropping the zero."""
    fixture_server.route("/house-latest", load_fixture("congress_house_latest.json"))
    rows = client.congressional.latest_house_disclosures(page=4_294_967_295, limit=0)

    assert fixture_server.requests[0].target == "/house-latest?page=4294967295&limit=0"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CongressionalTrade)
    assert row.symbol == "META"
    assert row.member_id == "M001217"
    assert row.district == "FL23"
    assert row.owner == ""
    assert row.disclosure_date == datetime.date(2026, 7, 27)
    assert row.transaction_date == datetime.date(2026, 6, 17)
    assert row.transaction_type == "Purchase"
    assert row.capital_gains_over_200_usd == "False"
    assert row.link == HOUSE_LINK


def test_latest_house_disclosures_with_limit_only(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_house_disclosures`` sends ``limit`` alone when ``page`` is omitted."""
    fixture_server.route("/house-latest", load_fixture("congress_house_latest.json"))
    rows = client.congressional.latest_house_disclosures(limit=250)

    assert fixture_server.requests[0].target == "/house-latest?limit=250"
    assert len(rows) == 1


def test_senate_trades_encodes_a_spaced_symbol_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``senate_trades`` form-encodes the ticker, then ``page``, then ``limit``."""
    fixture_server.route("/senate-trades", load_fixture("congress_senate_trades.json"))
    rows = client.congressional.senate_trades(SPACED_SYMBOL, page=1, limit=2)

    assert fixture_server.requests[0].target == f"/senate-trades?symbol={SPACED_SYMBOL_ENCODED}&page=1&limit=2"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CongressionalTrade)
    assert row.symbol == "AAPL"
    assert row.member_id == "W000802"
    assert row.first_name == "Sheldon"
    assert row.last_name == "Whitehouse"
    assert row.office == "Sheldon Whitehouse"
    assert row.district == "RI"
    assert row.owner == "Spouse"
    assert row.asset_description == "Apple Inc"
    assert row.disclosure_date == datetime.date(2026, 7, 8)
    assert row.transaction_date == datetime.date(2026, 6, 24)
    assert row.amount == "$15,001 - $50,000"


def test_senate_trades_with_symbol_only(client: Any, fixture_server: FixtureServer) -> None:
    """``senate_trades`` sends only ``symbol`` when neither page nor limit is set."""
    fixture_server.route("/senate-trades", load_fixture("congress_senate_trades.json"))
    rows = client.congressional.senate_trades("AAPL")

    assert fixture_server.requests[0].target == "/senate-trades?symbol=AAPL"
    assert fixture_server.requests[0].query == {"symbol": ["AAPL"]}
    assert len(rows) == 1


def test_senate_trades_by_name_form_encodes_the_space(client: Any, fixture_server: FixtureServer) -> None:
    """``senate_trades_by_name`` maps to ``/senate-trades-by-name`` with ``name`` alone."""
    fixture_server.route("/senate-trades-by-name", load_fixture("congress_senate_trades_by_name.json"))
    rows = client.congressional.senate_trades_by_name("Jerry Moran")

    assert fixture_server.requests[0].target == "/senate-trades-by-name?name=Jerry+Moran"
    assert fixture_server.requests[0].query == {"name": ["Jerry Moran"]}
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CongressionalTrade)
    assert row.symbol == ""
    assert row.member_id == "M000934"
    assert row.first_name == "Jerry"
    assert row.last_name == "Moran"
    assert row.district == "KS"
    assert row.asset_description == "Berkshire Hathaway Inc"
    assert row.disclosure_date == datetime.date(2026, 7, 21)
    assert row.transaction_date == datetime.date(2026, 6, 23)
    assert row.transaction_type == "Purchase"


def test_senate_trades_by_member_id_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``senate_trades_by_member_id`` encodes ``page``, ``limit``, then ``senateID`` from ``member_id``."""
    fixture_server.route("/senate-trades-by-id", load_fixture("congress_senate_trades_by_id.json"))
    rows = client.congressional.senate_trades_by_member_id(page=3, limit=4, member_id="M001242")

    assert fixture_server.requests[0].target == "/senate-trades-by-id?page=3&limit=4&senateID=M001242"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CongressionalTrade)
    assert row.symbol == "CM"
    assert row.member_id == "M001242"
    assert row.asset_type == "Corporate Bond"
    assert row.disclosure_date == datetime.date(2026, 7, 24)
    assert row.transaction_date == datetime.date(2026, 6, 22)
    assert row.capital_gains_over_200_usd == "False"


def test_senate_trades_by_member_id_with_member_id_only(client: Any, fixture_server: FixtureServer) -> None:
    """``member_id`` is independently optional and is sent as ``senateID`` alone."""
    fixture_server.route("/senate-trades-by-id", load_fixture("congress_senate_trades_by_id.json"))
    rows = client.congressional.senate_trades_by_member_id(member_id="M001242")

    assert fixture_server.requests[0].target == "/senate-trades-by-id?senateID=M001242"
    assert fixture_server.requests[0].query == {"senateID": ["M001242"]}
    assert len(rows) == 1


def test_house_trades_with_symbol_only(client: Any, fixture_server: FixtureServer) -> None:
    """``house_trades`` maps to ``/house-trades`` and sends the symbol alone."""
    fixture_server.route("/house-trades", load_fixture("congress_house_trades.json"))
    rows = client.congressional.house_trades("AAPL")

    assert fixture_server.requests[0].target == "/house-trades?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CongressionalTrade)
    assert row.symbol == "AAPL"
    assert row.member_id == "C001120"
    assert row.first_name == "Dan"
    assert row.last_name == "Crenshaw"
    assert row.district == "TX02"
    assert row.disclosure_date == datetime.date(2026, 7, 17)
    assert row.transaction_date == datetime.date(2026, 6, 1)
    assert row.transaction_type == "Sale"
    assert row.link == "https://disclosures-clerk.house.gov/public_disc/ptr-pdfs/2026/20035024.pdf"


def test_house_trades_with_page_and_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``house_trades`` encodes ``symbol``, ``page``, then ``limit``."""
    fixture_server.route("/house-trades", load_fixture("congress_house_trades.json"))
    rows = client.congressional.house_trades("AAPL", page=0, limit=250)

    assert fixture_server.requests[0].target == "/house-trades?symbol=AAPL&page=0&limit=250"
    assert len(rows) == 1


def test_house_trades_by_name_keeps_the_trailing_period(client: Any, fixture_server: FixtureServer) -> None:
    """``house_trades_by_name`` form-encodes the space and leaves the period untouched."""
    fixture_server.route("/house-trades-by-name", load_fixture("congress_house_trades_by_name.json"))
    rows = client.congressional.house_trades_by_name("James A.")

    assert fixture_server.requests[0].target == "/house-trades-by-name?name=James+A."
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CongressionalTrade)
    assert row.symbol == "BAC"
    assert row.member_id == "H001047"
    assert row.first_name == "James A."
    assert row.last_name == "Himes"
    assert row.office == "James A. Himes"
    assert row.owner == "Joint"
    assert row.disclosure_date == datetime.date(2026, 7, 21)
    assert row.transaction_date == datetime.date(2026, 7, 20)


def test_house_trades_by_member_id_without_options_sends_the_bare_path(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``house_trades_by_member_id`` with nothing set requests the bare ``/house-trades-by-id``."""
    fixture_server.route("/house-trades-by-id", load_fixture("congress_house_trades_by_id.json"))
    rows = client.congressional.house_trades_by_member_id()

    assert fixture_server.requests[0].target == "/house-trades-by-id"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CongressionalTrade)
    assert row.symbol == "MNST"
    assert row.member_id == "M001217"
    assert row.asset_description == "Monster Beverage Corp (2)"
    assert row.disclosure_date == datetime.date(2026, 7, 27)
    assert row.transaction_date == datetime.date(2026, 6, 17)
    assert row.transaction_type == "Purchase"


def test_house_trades_by_member_id_with_page_and_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``house_trades_by_member_id`` encodes ``page`` then ``limit`` without a member ID."""
    fixture_server.route("/house-trades-by-id", load_fixture("congress_house_trades_by_id.json"))
    rows = client.congressional.house_trades_by_member_id(page=0, limit=100)

    assert fixture_server.requests[0].target == "/house-trades-by-id?page=0&limit=100"
    assert "senateID=" not in fixture_server.requests[0].raw_query
    assert len(rows) == 1


def test_trade_feeds_decode_an_empty_body(client: Any, fixture_server: FixtureServer) -> None:
    """An empty JSON array decodes to an empty list for the trade row type."""
    fixture_server.route("/senate-latest", [])
    rows = client.congressional.latest_senate_disclosures()

    assert fixture_server.requests[0].target == "/senate-latest"
    assert rows == []
