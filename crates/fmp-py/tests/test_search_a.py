"""Runtime contract of ``client.search`` for the six search methods.

One test per method routes the documented fixture body, calls the method with
one argument shape, and asserts the exact request target plus a few typed
fields. The expected targets are the ones the Rust ``search_endpoints.rs``
proxy test pins: ``query`` then ``limit`` then ``exchange`` for the symbol and
name searches (form-encoded spaces and ``/``), ``cik`` then ``limit`` for the
CIK search, a single identifier for CUSIP, ISIN, and exchange variants (with
``^`` encoded as ``%5E``). The domain's only temporal field is
``ExchangeVariant.ipo_date`` (a ``datetime.date``); it has no
``datetime.datetime`` column. The negative cases and the error mapping live in
``test_search_b.py``.
"""

import datetime
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.search import (
    CikSearchResult,
    CusipSearchResult,
    ExchangeVariant,
    IsinSearchResult,
    NameSearchResult,
    SearchNamespace,
    SymbolSearchResult,
)

SPACED_QUERY = "Apple / Class A"
SPACED_QUERY_ENCODED = "Apple+%2F+Class+A"
U32_MAX = 4_294_967_295


def test_search_namespace_is_the_generated_type(client: Any) -> None:
    """``client.search`` is a ``SearchNamespace`` exposing all six methods."""
    assert isinstance(client.search, SearchNamespace)
    expected = {"symbol", "name", "cik", "cusip", "isin", "exchange_variants"}
    assert expected <= {name for name in dir(client.search) if not name.startswith("_")}


def test_symbol_with_limit_and_exchange(client: Any, fixture_server: FixtureServer) -> None:
    """``symbol`` encodes ``query`` then ``limit`` then ``exchange`` with form-encoded spaces."""
    fixture_server.route("/search-symbol", load_fixture("search_symbol.json"))
    rows = client.search.symbol(SPACED_QUERY, limit=U32_MAX, exchange="NASDAQ Global")

    assert (
        fixture_server.requests[0].target
        == f"/search-symbol?query={SPACED_QUERY_ENCODED}&limit={U32_MAX}&exchange=NASDAQ+Global"
    )
    assert fixture_server.requests[0].query == {
        "query": [SPACED_QUERY],
        "limit": [str(U32_MAX)],
        "exchange": ["NASDAQ Global"],
    }
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SymbolSearchResult)
    assert row.symbol == "AAPL"
    assert row.name == "Apple Inc."
    assert row.currency == "USD"
    assert row.exchange_full_name == "NASDAQ Global Select"
    assert row.exchange == "NASDAQ"


def test_symbol_with_query_only(client: Any, fixture_server: FixtureServer) -> None:
    """``symbol`` with only the search term sends ``query`` alone."""
    fixture_server.route("/search-symbol", load_fixture("search_symbol.json"))
    rows = client.search.symbol("AAPL")

    assert fixture_server.requests[0].target == "/search-symbol?query=AAPL"
    assert len(rows) == 1
    assert rows[0].symbol == "AAPL"


def test_symbol_with_exchange_only(client: Any, fixture_server: FixtureServer) -> None:
    """``symbol`` with ``exchange`` alone skips the ``limit`` parameter."""
    fixture_server.route("/search-symbol", load_fixture("search_symbol.json"))
    client.search.symbol("AAPL", exchange="NASDAQ")

    assert fixture_server.requests[0].target == "/search-symbol?query=AAPL&exchange=NASDAQ"


def test_symbol_preserves_an_empty_array(client: Any, fixture_server: FixtureServer) -> None:
    """``symbol`` returns ``[]`` for the provider's empty array."""
    fixture_server.route("/search-symbol", load_fixture("search_empty.json"))
    assert client.search.symbol("A") == []
    assert fixture_server.requests[0].target == "/search-symbol?query=A"


def test_symbol_decodes_multiple_rows_in_order(client: Any, fixture_server: FixtureServer) -> None:
    """``symbol`` keeps the provider's row order for a multi-row body."""
    fixture_server.route("/search-symbol", load_fixture("search_symbol_multiple.json"))
    rows = client.search.symbol("A")

    assert len(rows) == 2
    assert [row.symbol for row in rows] == ["000001.SZ", "^VIX"]
    assert rows[0].currency == "CNY"
    assert rows[0].exchange_full_name == "Shenzhen Stock Exchange"
    assert rows[1].exchange == "INDEX"
    assert rows[1].name == "CBOE Volatility Index"


def test_symbol_ignores_unknown_provider_fields(client: Any, fixture_server: FixtureServer) -> None:
    """``symbol`` tolerates an undocumented nested field without exposing it."""
    fixture_server.route("/search-symbol", load_fixture("search_symbol_unknown.json"))
    rows = client.search.symbol("A")

    assert len(rows) == 1
    assert rows[0].symbol == "AAPL"
    assert not hasattr(rows[0], "future_provider_field")


def test_name_with_query_only(client: Any, fixture_server: FixtureServer) -> None:
    """``name`` maps to ``/search-name`` and decodes the crypto listing row."""
    fixture_server.route("/search-name", load_fixture("search_name.json"))
    rows = client.search.name("AA")

    assert fixture_server.requests[0].target == "/search-name?query=AA"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, NameSearchResult)
    assert row.symbol == "AAGUSD"
    assert row.name == "AAG USD"
    assert row.currency == "USD"
    assert row.exchange_full_name == "CCC"
    assert row.exchange == "CRYPTO"


def test_name_with_limit_and_exchange(client: Any, fixture_server: FixtureServer) -> None:
    """``name`` encodes the same ``query``, ``limit``, ``exchange`` order as ``symbol``."""
    fixture_server.route("/search-name", load_fixture("search_name.json"))
    client.search.name(SPACED_QUERY, limit=5, exchange="CRYPTO")

    assert fixture_server.requests[0].target == f"/search-name?query={SPACED_QUERY_ENCODED}&limit=5&exchange=CRYPTO"


def test_name_with_limit_only(client: Any, fixture_server: FixtureServer) -> None:
    """``name`` with ``limit`` alone skips the ``exchange`` parameter."""
    fixture_server.route("/search-name", load_fixture("search_name.json"))
    client.search.name("AA", limit=0)

    assert fixture_server.requests[0].target == "/search-name?query=AA&limit=0"


def test_cik_with_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``cik`` encodes ``cik`` then ``limit`` and keeps the zero-padded CIK as a string."""
    fixture_server.route("/search-cik", load_fixture("search_cik.json"))
    rows = client.search.cik("0000320193", limit=50)

    assert fixture_server.requests[0].target == "/search-cik?cik=0000320193&limit=50"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CikSearchResult)
    assert row.symbol == "AAPL"
    assert row.company_name == "Apple Inc."
    assert row.cik == "0000320193"
    assert row.exchange_full_name == "NASDAQ Global Select"
    assert row.exchange == "NASDAQ"
    assert row.currency == "USD"


def test_cik_without_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``cik`` with nothing else set sends only ``cik``."""
    fixture_server.route("/search-cik", load_fixture("search_cik.json"))
    rows = client.search.cik("320193")

    assert fixture_server.requests[0].target == "/search-cik?cik=320193"
    assert fixture_server.requests[0].query == {"cik": ["320193"]}
    assert len(rows) == 1


def test_cusip_encodes_the_identifier(client: Any, fixture_server: FixtureServer) -> None:
    """``cusip`` maps to ``/search-cusip`` and decodes the large market cap as an ``int``."""
    fixture_server.route("/search-cusip", load_fixture("search_cusip.json"))
    rows = client.search.cusip("037833100")

    assert fixture_server.requests[0].target == "/search-cusip?cusip=037833100"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CusipSearchResult)
    assert row.symbol == "APC.F"
    assert row.company_name == "Apple Inc."
    assert row.cusip == "037833100"
    assert row.market_cap == 4_227_021_056_800
    assert isinstance(row.market_cap, int)


def test_isin_encodes_the_identifier(client: Any, fixture_server: FixtureServer) -> None:
    """``isin`` maps to ``/search-isin`` and decodes the ``name`` wire key."""
    fixture_server.route("/search-isin", load_fixture("search_isin.json"))
    rows = client.search.isin("US0378331005")

    assert fixture_server.requests[0].target == "/search-isin?isin=US0378331005"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, IsinSearchResult)
    assert row.symbol == "AAPL"
    assert row.name == "Apple Inc."
    assert row.isin == "US0378331005"
    assert row.market_cap == 4_874_072_686_740


def test_exchange_variants_encodes_the_caret(client: Any, fixture_server: FixtureServer) -> None:
    """``exchange_variants`` percent-encodes ``^`` and decodes the profile-shaped row."""
    fixture_server.route("/search-exchange-variants", load_fixture("search_exchange_variants.json"))
    rows = client.search.exchange_variants("^VIX")

    assert fixture_server.requests[0].target == "/search-exchange-variants?symbol=%5EVIX"
    assert fixture_server.requests[0].query == {"symbol": ["^VIX"]}
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, ExchangeVariant)
    assert row.symbol == "AAPL"
    assert row.company_name == "Apple Inc."
    assert row.ipo_date == datetime.date(1980, 12, 12)
    assert isinstance(row.ipo_date, datetime.date)
    assert row.price == pytest.approx(331.85501)
    assert row.beta == pytest.approx(1.097)
    assert row.changes == pytest.approx(-6.33498)
    assert row.dcf == pytest.approx(140.70269296445176)
    assert row.last_div == pytest.approx(1.05)
    assert row.market_cap == 4_874_072_686_740
    assert row.vol_avg == 55_309_000
    assert row.cik == "0000320193"
    assert row.isin == "US0378331005"
    assert row.cusip == "037833100"
    assert row.exchange == "NASDAQ Global Select"
    assert row.exchange_short_name == "NASDAQ"
    assert row.country == "US"
    assert row.full_time_employees == "166000"
    assert row.range == "201.5-344.57"
    assert row.is_actively_trading is True
    assert row.is_etf is False
    assert row.is_adr is False
    assert row.is_fund is False
    assert row.default_image is False


@pytest.mark.parametrize(
    ("method", "argument", "path"),
    [
        pytest.param("symbol", "AAPL", "/search-symbol", id="symbol"),
        pytest.param("name", "Apple", "/search-name", id="name"),
        pytest.param("cik", "320193", "/search-cik", id="cik"),
        pytest.param("cusip", "037833100", "/search-cusip", id="cusip"),
        pytest.param("isin", "US0378331005", "/search-isin", id="isin"),
        pytest.param("exchange_variants", "AAPL", "/search-exchange-variants", id="exchange-variants"),
    ],
)
def test_every_method_preserves_an_empty_array(
    client: Any, fixture_server: FixtureServer, method: str, argument: str, path: str
) -> None:
    """Each method returns ``[]`` for the provider's empty array on its own path."""
    fixture_server.route(path, load_fixture("search_empty.json"))
    assert getattr(client.search, method)(argument) == []
    assert fixture_server.requests[0].path == path
