"""Runtime contract of ``client.news`` for the four ticker-list searches and the negatives.

Covers ``search_press_releases``, ``search_stock_news``, ``search_crypto_news``,
and ``search_forex_news``: one test per method routes the documented fixture
body, calls the method with one argument shape, and asserts the exact request
target plus a few typed fields. The expected targets are the ones the Rust
``news_search_endpoints.rs`` test pins. The negative cases cover every argument
kind the domain uses (``ticker_list``, ``date``, ``page``, ``limit``) plus the
status and decode error mapping. The authored feed and the five latest feeds
live in ``test_news_a.py``.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.news import NewsArticle

FROM_2026_01_27 = datetime.date(2026, 1, 27)
TO_2026_04_28 = datetime.date(2026, 4, 28)


def test_search_press_releases_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``search_press_releases`` encodes ``symbols`` first, form-encoding a spaced ticker."""
    fixture_server.route("/news/press-releases", load_fixture("search_press_releases.json"))
    rows = client.news.search_press_releases(
        ["AAPL", "BRK.B / Class A"], from_=FROM_2026_01_27, to=TO_2026_04_28, page=0, limit=251
    )

    assert (
        fixture_server.requests[0].target
        == "/news/press-releases?symbols=AAPL%2CBRK.B+%2F+Class+A&from=2026-01-27&to=2026-04-28&page=0&limit=251"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, NewsArticle)
    assert row.symbol == "AAPL"
    assert row.published_date == datetime.datetime(2026, 7, 28, 8, 15, 0)
    assert row.publisher == "Business Wire"
    assert row.title == "Apple Upgrade launches in the United States"
    assert row.text.count("®") == 6
    assert "“At Apple" in row.text


def test_search_press_releases_with_symbols_only(client: Any, fixture_server: FixtureServer) -> None:
    """``search_press_releases`` sends only the comma-joined ``symbols`` without optionals."""
    fixture_server.route("/news/press-releases", load_fixture("search_press_releases.json"))
    rows = client.news.search_press_releases(["AAPL", "MSFT"])

    assert fixture_server.requests[0].target == "/news/press-releases?symbols=AAPL%2CMSFT"
    assert fixture_server.requests[0].query == {"symbols": ["AAPL,MSFT"]}
    assert len(rows) == 1


def test_search_stock_news_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``search_stock_news`` keeps the ticker order and decodes the YouTube-hosted row."""
    fixture_server.route("/news/stock", load_fixture("search_stock_news.json"))
    rows = client.news.search_stock_news(
        ["MSFT", "BRK.B / Class A"], from_="2026-01-27", to="2026-04-28", page=0, limit=251
    )

    assert (
        fixture_server.requests[0].target
        == "/news/stock?symbols=MSFT%2CBRK.B+%2F+Class+A&from=2026-01-27&to=2026-04-28&page=0&limit=251"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, NewsArticle)
    assert row.symbol == "AAPL"
    assert row.published_date == datetime.datetime(2026, 7, 30, 12, 36, 38)
    assert row.publisher == "CNBC Television"
    assert row.site == "youtube.com"
    assert row.url == "https://www.youtube.com/watch?v=ZKMD80U8dRM"


def test_search_stock_news_with_from_only(client: Any, fixture_server: FixtureServer) -> None:
    """``search_stock_news`` sends ``symbols`` then ``from`` when only the start date is given."""
    fixture_server.route("/news/stock", load_fixture("search_stock_news.json"))
    client.news.search_stock_news(["AAPL", "MSFT"], from_=FROM_2026_01_27)

    assert fixture_server.requests[0].target == "/news/stock?symbols=AAPL%2CMSFT&from=2026-01-27"
    assert "from_" not in fixture_server.requests[0].raw_query


def test_search_stock_news_accepts_a_single_symbol_string(client: Any, fixture_server: FixtureServer) -> None:
    """``search_stock_news`` takes one ticker as a bare ``str`` and sends it under ``symbols``."""
    fixture_server.route("/news/stock", load_fixture("search_stock_news.json"))
    rows = client.news.search_stock_news("AAPL")

    assert fixture_server.requests[0].target == "/news/stock?symbols=AAPL"
    assert len(rows) == 1


def test_search_crypto_news_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``search_crypto_news`` form-encodes the slash inside a pair and decodes the crypto row."""
    fixture_server.route("/news/crypto", load_fixture("search_crypto_news.json"))
    rows = client.news.search_crypto_news(
        ["BTCUSD", "ETH/USD"], from_=FROM_2026_01_27, to=TO_2026_04_28, page=0, limit=251
    )

    assert (
        fixture_server.requests[0].target
        == "/news/crypto?symbols=BTCUSD%2CETH%2FUSD&from=2026-01-27&to=2026-04-28&page=0&limit=251"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, NewsArticle)
    assert row.symbol == "BTCUSD"
    assert row.published_date == datetime.datetime(2026, 7, 30, 13, 8, 46)
    assert row.publisher == "AMBCrypto"
    assert row.title.startswith("Fidelity Bitcoin ETF posts $43.1 million in outflows")


def test_search_crypto_news_with_to_only(client: Any, fixture_server: FixtureServer) -> None:
    """``search_crypto_news`` sends ``symbols`` then ``to`` when only the end date is given."""
    fixture_server.route("/news/crypto", load_fixture("search_crypto_news.json"))
    client.news.search_crypto_news(["BTCUSD", "ETHUSD"], to=TO_2026_04_28)

    assert fixture_server.requests[0].target == "/news/crypto?symbols=BTCUSD%2CETHUSD&to=2026-04-28"


def test_search_forex_news_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``search_forex_news`` form-encodes the slash inside a pair and decodes the forex row."""
    fixture_server.route("/news/forex", load_fixture("search_forex_news.json"))
    rows = client.news.search_forex_news(
        ["EURUSD", "USD/JPY"], from_=FROM_2026_01_27, to=TO_2026_04_28, page=0, limit=251
    )

    assert (
        fixture_server.requests[0].target
        == "/news/forex?symbols=EURUSD%2CUSD%2FJPY&from=2026-01-27&to=2026-04-28&page=0&limit=251"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, NewsArticle)
    assert row.symbol == "EURUSD"
    assert row.published_date == datetime.datetime(2026, 7, 30, 13, 3, 11)
    assert row.publisher == "FXEmpire"
    assert row.text.startswith("BoJ intervened to support the yen")


def test_search_forex_news_with_page_zero_only(client: Any, fixture_server: FixtureServer) -> None:
    """``search_forex_news`` sends ``symbols`` then ``page=0`` and rejects a positional date."""
    fixture_server.route("/news/forex", load_fixture("search_forex_news.json"))
    client.news.search_forex_news(["EURUSD", "USDJPY"], page=0)

    assert fixture_server.requests[0].target == "/news/forex?symbols=EURUSD%2CUSDJPY&page=0"
    with pytest.raises(TypeError):
        client.news.search_forex_news(["EURUSD"], FROM_2026_01_27)
    with pytest.raises(TypeError):
        client.news.search_forex_news()


def test_search_feeds_decode_an_empty_array(client: Any, fixture_server: FixtureServer) -> None:
    """A documented empty body decodes to an empty list on every search feed."""
    fixture_server.route("/news/press-releases", [])
    fixture_server.route("/news/forex", [])

    assert client.news.search_press_releases("AAPL") == []
    assert client.news.search_forex_news("EURUSD", limit=0) == []
    assert [request.target for request in fixture_server.requests] == [
        "/news/press-releases?symbols=AAPL",
        "/news/forex?symbols=EURUSD&limit=0",
    ]


@pytest.mark.parametrize(
    ("symbols", "message"),
    [
        pytest.param([], "symbols: ticker list must contain at least one ticker", id="empty-list"),
        pytest.param(["AAPL", " "], "symbols[1]: value must not be empty or whitespace-only", id="blank-element"),
        pytest.param("AAPL,MSFT", "symbols: ticker must not contain a comma", id="comma-in-str"),
    ],
)
def test_invalid_symbols_name_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, symbols: Any, message: str
) -> None:
    """``ticker_list`` validation fails locally and names ``symbols`` (with the index)."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.news.search_stock_news(symbols)
    error = raised.value
    assert str(error).startswith(message)
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("method", "keyword", "value", "message"),
    [
        pytest.param(
            "latest_general_news", "from_", "27/01/2026", "from_: value must be a valid YYYY-MM-DD date", id="from"
        ),
        pytest.param("search_crypto_news", "to", "2026-13-01", "to: value must be a valid YYYY-MM-DD date", id="to"),
        pytest.param("articles", "page", -1, "page: must be an integer from 0 through 4294967295", id="page"),
        pytest.param("latest_forex_news", "limit", -1, "limit: ", id="limit"),
        pytest.param("search_press_releases", "page", 4_294_967_296, "page: ", id="page-overflow"),
    ],
)
def test_invalid_optionals_name_the_argument(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    keyword: str,
    value: Any,
    message: str,
) -> None:
    """Each optional kind family fails locally with ``FmpValidationError`` prefixed by the keyword."""
    kwargs: dict[str, Any] = {keyword: value}
    if method.startswith("search_"):
        kwargs = {"symbols": ["AAPL"], keyword: value}
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.news, method)(**kwargs)
    error = raised.value
    assert str(error).startswith(message)
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_status_error_carries_the_news_endpoint_id(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on a nested news path maps to ``FmpStatusError`` with the endpoint id."""
    fixture_server.route("/news/stock", {"error": "denied"}, status=401)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.news.search_stock_news(["AAPL"])
    error = raised.value
    assert error.endpoint == "news/stock"
    assert error.status == 401
    assert error.body == '{"error": "denied"}'


def test_non_json_body_is_a_decode_error(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-JSON body on the authored feed maps to ``FmpDecodeError``."""
    fixture_server.route("/fmp-articles", b"not-json " * 40)
    with pytest.raises(errors.FmpDecodeError):
        client.news.articles()
