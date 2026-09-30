"""Runtime contract of the 13 ``client.quote`` methods added by PY2-12.

``full``, ``short``, and ``mutual_funds`` are covered in ``test_client.py``.
One test per method routes the documented fixture body, calls the method
with one argument shape, and asserts the exact request target plus a few
typed fields. The expected targets are the ones the Rust
``quote_single_endpoints.rs``, ``quote_batch_endpoints.rs``, and
``quote_universe_endpoints.rs`` tests pin. No quote model carries a date
field. Every ``timestamp`` is a plain ``int`` in Python, but the unit differs
by model exactly as the Rust newtypes record it: ``Quote.timestamp`` is Unix
seconds (``UnixSeconds``), while ``AftermarketTrade.timestamp`` and
``AftermarketQuote.timestamp`` are Unix milliseconds (``UnixMilliseconds``).
"""

from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.quote import AftermarketQuote, AftermarketTrade, Quote, QuoteNamespace, QuoteShort, StockPriceChange

AFTERMARKET_TIMESTAMP = 1_785_430_813_000
SPACED_SYMBOL = "BRK.B / Class A"
SPACED_SYMBOL_ENCODED = "BRK.B+%2F+Class+A"


def test_quote_namespace_is_the_generated_type(client: Any) -> None:
    """``client.quote`` is the generated namespace class."""
    assert isinstance(client.quote, QuoteNamespace)


def test_aftermarket_trade_takes_one_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``aftermarket_trade`` maps to ``/aftermarket-trade`` with the symbol alone."""
    fixture_server.route("/aftermarket-trade", load_fixture("aftermarket_trade.json"))
    rows = client.quote.aftermarket_trade("AAPL")

    assert fixture_server.requests[0].target == "/aftermarket-trade?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, AftermarketTrade)
    assert row.symbol == "AAPL"
    assert row.price == pytest.approx(331.85999)
    assert row.trade_size == 16
    assert row.timestamp == AFTERMARKET_TIMESTAMP


def test_aftermarket_trade_null_trade_size_is_none(client: Any, fixture_server: FixtureServer) -> None:
    """A ``null`` provider ``tradeSize`` surfaces as ``None``."""
    fixture_server.route("/aftermarket-trade", load_fixture("aftermarket_trade_synthetic.json"))
    rows = client.quote.aftermarket_trade("000001.SZ")

    assert len(rows) == 3
    assert rows[2].trade_size is None


def test_aftermarket_quote_takes_one_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``aftermarket_quote`` decodes the bid-and-ask row for one symbol."""
    fixture_server.route("/aftermarket-quote", load_fixture("aftermarket_quote.json"))
    rows = client.quote.aftermarket_quote("AAPL")

    assert fixture_server.requests[0].target == "/aftermarket-quote?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, AftermarketQuote)
    assert row.symbol == "AAPL"
    assert row.bid_price == pytest.approx(331.85)
    assert row.bid_size == 16
    assert row.ask_price == pytest.approx(331.88)
    assert row.ask_size == 40
    assert row.volume == 28_718_455
    assert row.timestamp == AFTERMARKET_TIMESTAMP


def test_stock_price_change_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``stock_price_change`` form-encodes the ticker and maps the ``1D``..``max`` keys."""
    fixture_server.route("/stock-price-change", load_fixture("stock_price_change.json"))
    rows = client.quote.stock_price_change(SPACED_SYMBOL)

    assert fixture_server.requests[0].target == f"/stock-price-change?symbol={SPACED_SYMBOL_ENCODED}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockPriceChange)
    assert row.symbol == "AAPL"
    assert row.one_day == pytest.approx(-1.8732)
    assert row.five_days == pytest.approx(3.12782)
    assert row.year_to_date == pytest.approx(22.06835)
    assert row.ten_years == pytest.approx(1151.81068)
    assert row.max == pytest.approx(258_454.74094)


def test_batch_quote_joins_a_symbol_list_with_a_comma(client: Any, fixture_server: FixtureServer) -> None:
    """``batch`` sends ``symbols=`` as a comma-joined, form-encoded list."""
    fixture_server.route("/batch-quote", load_fixture("quote.json"))
    rows = client.quote.batch(["AAPL", "MSFT"])

    assert fixture_server.requests[0].target == "/batch-quote?symbols=AAPL%2CMSFT"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, Quote)
    assert row.symbol == "AAPL"
    assert row.name == "Apple Inc."
    assert row.price == pytest.approx(331.85501)
    assert row.market_cap == 4_874_072_686_740
    assert row.timestamp == 1_785_430_812


def test_batch_quote_short_accepts_a_bare_string(client: Any, fixture_server: FixtureServer) -> None:
    """A bare ``str`` is one ticker, never split on commas."""
    fixture_server.route("/batch-quote-short", load_fixture("quote_short_multiple.json"))
    rows = client.quote.batch_short("AAPL")

    assert fixture_server.requests[0].target == "/batch-quote-short?symbols=AAPL"
    assert len(rows) == 2
    assert all(isinstance(row, QuoteShort) for row in rows)
    assert [row.symbol for row in rows] == ["000001.SZ", "^VIX"]
    assert rows[0].volume == 4_294_967_296
    assert rows[1].price == pytest.approx(18.75)
    assert rows[1].change is None
    assert rows[1].volume is None


def test_batch_aftermarket_trade_preserves_symbol_order(client: Any, fixture_server: FixtureServer) -> None:
    """``batch_aftermarket_trade`` keeps request order and encodes every ticker."""
    fixture_server.route("/batch-aftermarket-trade", load_fixture("aftermarket_trade.json"))
    rows = client.quote.batch_aftermarket_trade([SPACED_SYMBOL, "^VIX", "000001.SZ"])

    expected = f"/batch-aftermarket-trade?symbols={SPACED_SYMBOL_ENCODED}%2C%5EVIX%2C000001.SZ"
    assert fixture_server.requests[0].target == expected
    assert len(rows) == 1
    assert isinstance(rows[0], AftermarketTrade)
    assert rows[0].trade_size == 16
    assert rows[0].timestamp == AFTERMARKET_TIMESTAMP


def test_batch_aftermarket_quote_decodes_bid_and_ask(client: Any, fixture_server: FixtureServer) -> None:
    """``batch_aftermarket_quote`` maps to ``/batch-aftermarket-quote``."""
    fixture_server.route("/batch-aftermarket-quote", load_fixture("aftermarket_quote.json"))
    rows = client.quote.batch_aftermarket_quote(["AAPL", "MSFT"])

    assert fixture_server.requests[0].target == "/batch-aftermarket-quote?symbols=AAPL%2CMSFT"
    assert len(rows) == 1
    assert isinstance(rows[0], AftermarketQuote)
    assert rows[0].ask_price == pytest.approx(331.88)
    assert rows[0].bid_size == 16
    assert rows[0].timestamp == AFTERMARKET_TIMESTAMP


def test_exchange_adds_the_fixed_short_flag(client: Any, fixture_server: FixtureServer) -> None:
    """``exchange`` form-encodes the exchange code; libfmp always appends ``short=true``."""
    fixture_server.route("/batch-exchange-quote", load_fixture("quote_exchange_short.json"))
    rows = client.quote.exchange("NASDAQ Global / Select")

    assert fixture_server.requests[0].target == "/batch-exchange-quote?exchange=NASDAQ+Global+%2F+Select&short=true"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, QuoteShort)
    assert row.symbol == "AAACX"
    assert row.price == pytest.approx(6.61)
    assert row.change == pytest.approx(0.0)
    assert row.volume == 0


@pytest.mark.parametrize(
    ("method", "path", "fixture", "symbol", "field", "value"),
    [
        pytest.param("etfs", "/batch-etf-quotes", "quote_etf_short.json", "P60.SI", "volume", 1, id="etfs"),
        pytest.param(
            "commodities",
            "/batch-commodity-quotes",
            "quote_commodity_short.json",
            "DCUSD",
            "change",
            0.02,
            id="commodities",
        ),
        pytest.param(
            "cryptocurrencies", "/batch-crypto-quotes", "quote_crypto_short.json", "00USD", "price", 0.0102, id="crypto"
        ),
        pytest.param("forex", "/batch-forex-quotes", "quote_forex_short.json", "AEDAUD", "price", 0.38716, id="forex"),
        pytest.param(
            "indexes", "/batch-index-quotes", "quote_index_short.json", "^SPROME10", "change", 44.67, id="indexes"
        ),
    ],
)
def test_universe_methods_take_no_arguments(
    client: Any,
    fixture_server: FixtureServer,
    method: str,
    path: str,
    fixture: str,
    symbol: str,
    field: str,
    value: float,
) -> None:
    """Each universe method hits its own path with only the fixed ``short=true``."""
    fixture_server.route(path, load_fixture(fixture))
    rows = getattr(client.quote, method)()

    assert fixture_server.requests[0].target == f"{path}?short=true"
    assert len(rows) == 1
    assert isinstance(rows[0], QuoteShort)
    assert rows[0].symbol == symbol
    assert getattr(rows[0], field) == pytest.approx(value)
    with pytest.raises(TypeError):
        getattr(client.quote, method)("AAPL")


@pytest.mark.parametrize(
    ("symbols", "message"),
    [
        pytest.param([], "symbols: ticker list must contain at least one ticker", id="empty-list"),
        pytest.param(["AAPL", " "], "symbols[1]: value must not be empty or whitespace-only", id="blank-element"),
        pytest.param("AAPL,MSFT", "symbols: ticker must not contain a comma", id="comma-in-str"),
    ],
)
def test_invalid_symbol_lists_name_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, symbols: Any, message: str
) -> None:
    """``ticker_list`` validation fails locally and names ``symbols`` (with the index)."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.quote.batch(symbols)
    error = raised.value
    assert str(error) == message
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_invalid_exchange_code_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """An empty exchange code is rejected before any request."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.quote.exchange("   ")
    assert str(raised.value) == "exchange: value must not be empty or whitespace-only"
    assert fixture_server.requests == []


def test_batch_status_error_names_the_batch_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on a batch call carries the batch endpoint id."""
    fixture_server.route("/batch-quote-short", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.quote.batch_short(["AAPL", "MSFT"])
    error = raised.value
    assert error.endpoint == "batch-quote-short"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_aftermarket_trade_decodes_a_fractional_trade_size(client: Any, fixture_server: FixtureServer) -> None:
    """Issue #340: a fractional trade size decodes as ``float``."""
    fixture_server.route("/aftermarket-trade", load_fixture("aftermarket_trade_fractional_synthetic.json"))
    rows = client.quote.aftermarket_trade("AAPL")

    assert len(rows) == 1
    assert isinstance(rows[0].trade_size, float)
    assert rows[0].trade_size == 16.5
