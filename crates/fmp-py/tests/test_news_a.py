"""Runtime contract of ``client.news`` for the authored feed and the five latest feeds.

Covers ``fmp_articles``, ``latest_general_news``, ``latest_press_releases``,
``latest_stock_news``, ``latest_crypto_news``, and ``latest_forex_news``: one
test per method routes the documented fixture body, calls the method with one
argument shape, and asserts the exact request target plus a few typed fields
(including the ``datetime.datetime`` ones). The expected targets are the ones
the Rust ``news_authored_general_press_endpoints.rs`` and
``news_market_latest_endpoints.rs`` tests pin. The four ticker-list searches
and the negative cases live in ``test_news_b.py``.
"""

import datetime
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.news import FmpArticle, NewsArticle, NewsNamespace

FROM_2026_01_27 = datetime.date(2026, 1, 27)
TO_2026_04_28 = datetime.date(2026, 4, 28)


def test_news_namespace_is_the_generated_type(client: Any) -> None:
    """``client.news`` is the generated flat namespace class."""
    assert isinstance(client.news, NewsNamespace)


def test_fmp_articles_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``fmp_articles`` encodes ``page`` then ``limit`` and decodes the editorial row."""
    fixture_server.route("/fmp-articles", load_fixture("fmp_articles.json"))
    rows = client.news.fmp_articles(page=0, limit=20)

    assert fixture_server.requests[0].target == "/fmp-articles?page=0&limit=20"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, FmpArticle)
    assert row.title.startswith("Centerra Gold (NYSE:CGAU)")
    assert row.date == datetime.datetime(2026, 7, 30, 16, 11, 45)
    assert row.tickers == "NYSE:CGAU"
    assert row.author == "Andrew Wynn"
    assert row.site == "Financial Modeling Prep"
    assert row.content.startswith("<ul>\n    <li><strong>")
    assert row.link.startswith("https://financialmodelingprep.com/market-news/")


def test_fmp_articles_without_options(client: Any, fixture_server: FixtureServer) -> None:
    """``fmp_articles`` sends a bare path when neither page nor limit is given."""
    fixture_server.route("/fmp-articles", load_fixture("fmp_articles.json"))
    rows = client.news.fmp_articles()

    assert fixture_server.requests[0].target == "/fmp-articles"
    assert len(rows) == 1
    assert isinstance(rows[0], FmpArticle)


def test_fmp_articles_with_page_only(client: Any, fixture_server: FixtureServer) -> None:
    """``fmp_articles`` sends ``page`` without ``limit`` and rejects a positional page."""
    fixture_server.route("/fmp-articles", load_fixture("fmp_articles.json"))
    client.news.fmp_articles(page=0)

    assert fixture_server.requests[0].target == "/fmp-articles?page=0"
    with pytest.raises(TypeError):
        client.news.fmp_articles(0)


def test_latest_general_news_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_general_news`` encodes ``from``, ``to``, ``page``, ``limit`` and keeps the null symbol."""
    fixture_server.route("/news/general-latest", load_fixture("latest_general_news.json"))
    rows = client.news.latest_general_news(from_=FROM_2026_01_27, to=TO_2026_04_28, page=100, limit=251)

    assert fixture_server.requests[0].target == "/news/general-latest?from=2026-01-27&to=2026-04-28&page=100&limit=251"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, NewsArticle)
    assert row.symbol is None
    assert row.published_date == datetime.datetime(2026, 7, 30, 12, 53, 41)
    assert row.publisher == "Seeking Alpha"
    assert row.site == "seekingalpha.com"
    assert row.text == "Polaris Renewable Energy Inc. (PIF:CA) Q2 2026 Earnings Call Transcript"
    assert row.url.startswith("https://seekingalpha.com/article/")


def test_latest_general_news_with_from_only(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_general_news`` takes ``from_`` as an ISO string and sends only ``from``."""
    fixture_server.route("/news/general-latest", load_fixture("latest_general_news.json"))
    rows = client.news.latest_general_news(from_="2026-01-27")

    assert fixture_server.requests[0].target == "/news/general-latest?from=2026-01-27"
    assert fixture_server.requests[0].query == {"from": ["2026-01-27"]}
    assert "from_" not in fixture_server.requests[0].raw_query
    assert len(rows) == 1


def test_latest_general_news_without_options(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_general_news`` sends a bare path with no arguments."""
    fixture_server.route("/news/general-latest", load_fixture("latest_general_news.json"))
    rows = client.news.latest_general_news()

    assert fixture_server.requests[0].target == "/news/general-latest"
    assert len(rows) == 1


def test_latest_press_releases_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_press_releases`` accepts a zero page and preserves the curly quotes in ``text``."""
    fixture_server.route("/news/press-releases-latest", load_fixture("latest_press_releases.json"))
    rows = client.news.latest_press_releases(from_=FROM_2026_01_27, to=TO_2026_04_28, page=0, limit=251)

    assert (
        fixture_server.requests[0].target
        == "/news/press-releases-latest?from=2026-01-27&to=2026-04-28&page=0&limit=251"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, NewsArticle)
    assert row.symbol == "RXT"
    assert row.published_date == datetime.datetime(2026, 7, 30, 13, 8, 0)
    assert row.publisher == "GlobeNewsWire"
    assert "“Rackspace”" in row.text
    assert row.title.endswith("Rackspace Technology, Inc. (RXT)")


def test_latest_press_releases_with_to_only(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_press_releases`` sends only ``to`` when only the end date is given."""
    fixture_server.route("/news/press-releases-latest", load_fixture("latest_press_releases.json"))
    rows = client.news.latest_press_releases(to=TO_2026_04_28)

    assert fixture_server.requests[0].target == "/news/press-releases-latest?to=2026-04-28"
    assert len(rows) == 1


def test_latest_stock_news_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_stock_news`` encodes the four optionals and keeps the query string of ``url``."""
    fixture_server.route("/news/stock-latest", load_fixture("latest_stock_news.json"))
    rows = client.news.latest_stock_news(from_="2026-01-27", to="2026-04-28", page=100, limit=251)

    assert fixture_server.requests[0].target == "/news/stock-latest?from=2026-01-27&to=2026-04-28&page=100&limit=251"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, NewsArticle)
    assert row.symbol == "KO"
    assert row.published_date == datetime.datetime(2026, 7, 30, 13, 15, 49)
    assert row.publisher == "Zacks Investment Research"
    assert row.title == "Coca-Cola's Momentum Builds After Strong Q2 Earnings: ETFs to Consider"
    assert row.url.endswith("?cid=CS-STOCKNEWSAPI-FT-etf_news_and_commentary-2964897")


def test_latest_stock_news_with_from_only(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_stock_news`` sends only ``from`` when only the start date is given."""
    fixture_server.route("/news/stock-latest", load_fixture("latest_stock_news.json"))
    client.news.latest_stock_news(from_=FROM_2026_01_27)

    assert fixture_server.requests[0].target == "/news/stock-latest?from=2026-01-27"


def test_latest_crypto_news_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_crypto_news`` encodes the four optionals and decodes the crypto row."""
    fixture_server.route("/news/crypto-latest", load_fixture("latest_crypto_news.json"))
    rows = client.news.latest_crypto_news(from_=FROM_2026_01_27, to=TO_2026_04_28, page=0, limit=251)

    assert fixture_server.requests[0].target == "/news/crypto-latest?from=2026-01-27&to=2026-04-28&page=0&limit=251"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, NewsArticle)
    assert row.symbol == "UNIUSD"
    assert row.published_date == datetime.datetime(2026, 7, 30, 13, 15, 17)
    assert row.publisher == "Crypto Briefing"
    assert row.url == "https://cryptobriefing.com/uniswap-launches-beta-tab-token-launches/"


def test_latest_crypto_news_with_to_only(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_crypto_news`` sends only ``to`` when only the end date is given."""
    fixture_server.route("/news/crypto-latest", load_fixture("latest_crypto_news.json"))
    client.news.latest_crypto_news(to="2026-04-28")

    assert fixture_server.requests[0].target == "/news/crypto-latest?to=2026-04-28"


def test_latest_forex_news_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_forex_news`` encodes the four optionals and decodes the forex row."""
    fixture_server.route("/news/forex-latest", load_fixture("latest_forex_news.json"))
    rows = client.news.latest_forex_news(from_=FROM_2026_01_27, to=TO_2026_04_28, page=0, limit=251)

    assert fixture_server.requests[0].target == "/news/forex-latest?from=2026-01-27&to=2026-04-28&page=0&limit=251"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, NewsArticle)
    assert row.symbol == "USDJPY"
    assert row.published_date == datetime.datetime(2026, 7, 30, 13, 3, 11)
    assert row.publisher == "FXEmpire"
    assert row.site == "fxempire.com"
    assert row.url.startswith("https://www.fxempire.com/forecasts/article/")


def test_latest_forex_news_with_page_zero_only(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_forex_news`` sends ``page=0`` on its own and rejects a positional argument."""
    fixture_server.route("/news/forex-latest", load_fixture("latest_forex_news.json"))
    client.news.latest_forex_news(page=0)

    assert fixture_server.requests[0].target == "/news/forex-latest?page=0"
    with pytest.raises(TypeError):
        client.news.latest_forex_news(FROM_2026_01_27)


def test_latest_feeds_decode_an_empty_array(client: Any, fixture_server: FixtureServer) -> None:
    """A documented empty body decodes to an empty list on the authored and latest feeds."""
    fixture_server.route("/fmp-articles", [])
    fixture_server.route("/news/general-latest", [])
    fixture_server.route("/news/stock-latest", [])

    assert client.news.fmp_articles() == []
    assert client.news.latest_general_news() == []
    assert client.news.latest_stock_news(limit=0) == []
    assert [request.target for request in fixture_server.requests] == [
        "/fmp-articles",
        "/news/general-latest",
        "/news/stock-latest?limit=0",
    ]
