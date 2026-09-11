"""Runtime contract of ``client.forex`` for the catalog, quote, and chart methods.

One test per method routes the documented fixture body, calls the method with
one argument shape, and asserts the exact request target plus a few typed
fields (including the ``datetime.date`` / ``datetime.datetime`` ones). The
expected targets are the ones the Rust ``asset_catalog_quote_endpoints.rs``
and ``asset_history_endpoints.rs`` tests pin. The quote and chart rows are the
cross-domain ``fmp.quote`` / ``fmp.chart`` models; only ``ForexPair`` lives in
``fmp.forex``. The negative cases live in ``test_forex_b.py``.
"""

import datetime
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.chart import StockChartFullBar, StockChartIntradayBar, StockChartLightBar
from fmp.forex import ForexNamespace, ForexPair
from fmp.quote import Quote, QuoteShort

EOD_FROM = datetime.date(2026, 1, 27)
EOD_TO = datetime.date(2026, 4, 27)
INTRADAY_FROM = datetime.date(2024, 1, 1)
INTRADAY_TO = datetime.date(2024, 3, 1)
EOD_QUERY = "symbol=EURUSD&from=2026-01-27&to=2026-04-27"
INTRADAY_QUERY = "symbol=EURUSD&from=2024-01-01&to=2024-03-01"


def test_forex_namespace_is_the_generated_type(client: Any) -> None:
    """``client.forex`` is the generated flat namespace class."""
    assert isinstance(client.forex, ForexNamespace)


def test_list_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``list`` maps to ``/forex-list`` with no query string and decodes the pair row."""
    fixture_server.route("/forex-list", load_fixture("forex_list.json"))
    rows = client.forex.list()

    assert fixture_server.requests[0].target == "/forex-list"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, ForexPair)
    assert row.symbol == "ARSMXN"
    assert row.from_currency == "ARS"
    assert row.to_currency == "MXN"
    assert row.from_name == "Argentine Peso"
    assert row.to_name == "Mexican Peso"


def test_quote_encodes_the_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``quote`` reuses the generic ``/quote`` route and decodes the full quote row."""
    fixture_server.route("/quote", load_fixture("forex_quote.json"))
    rows = client.forex.quote("EURUSD")

    assert fixture_server.requests[0].target == "/quote?symbol=EURUSD"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, Quote)
    assert row.symbol == "EURUSD"
    assert row.name == "EUR/USD"
    assert row.exchange == "FOREX"
    assert row.price == pytest.approx(1.15284)
    assert row.change_percentage == pytest.approx(0.55071)
    assert row.market_cap is None
    assert row.timestamp == 1_785_430_798


def test_quote_short_encodes_the_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``quote_short`` reuses the generic ``/quote-short`` route."""
    fixture_server.route("/quote-short", load_fixture("forex_quote_short.json"))
    rows = client.forex.quote_short("EURUSD")

    assert fixture_server.requests[0].target == "/quote-short?symbol=EURUSD"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, QuoteShort)
    assert row.symbol == "EURUSD"
    assert row.price == pytest.approx(1.15284)
    assert row.change == pytest.approx(0.006314)
    assert row.volume == 146_872


def test_chart_light_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_light`` encodes ``symbol``, ``from``, ``to`` in that order and decodes a date row."""
    fixture_server.route("/historical-price-eod/light", load_fixture("forex_chart_light.json"))
    rows = client.forex.chart_light("EURUSD", from_=EOD_FROM, to=EOD_TO)

    assert fixture_server.requests[0].target == f"/historical-price-eod/light?{EOD_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartLightBar)
    assert row.symbol == "EURUSD"
    assert row.date == datetime.date(2026, 7, 30)
    assert row.price == pytest.approx(1.15258)
    assert row.volume == 147_799


def test_chart_light_omits_to_independently(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_light`` accepts an ISO string for ``from_`` and leaves ``to`` out."""
    fixture_server.route("/historical-price-eod/light", load_fixture("forex_chart_light.json"))
    rows = client.forex.chart_light("EURUSD", from_="2026-01-27")

    assert fixture_server.requests[0].target == "/historical-price-eod/light?symbol=EURUSD&from=2026-01-27"
    assert len(rows) == 1


def test_chart_full_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_full`` maps to ``/historical-price-eod/full`` and decodes the OHLC row."""
    fixture_server.route("/historical-price-eod/full", load_fixture("forex_chart_full.json"))
    rows = client.forex.chart_full("EURUSD", from_="2026-01-27", to="2026-04-27")

    assert fixture_server.requests[0].target == f"/historical-price-eod/full?{EOD_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartFullBar)
    assert row.symbol == "EURUSD"
    assert row.date == datetime.date(2026, 7, 30)
    assert row.open == pytest.approx(1.14666)
    assert row.close == pytest.approx(1.15258)
    assert row.change_percent == pytest.approx(0.51628207)
    assert row.vwap == pytest.approx(1.15)
    assert row.volume == 147_799


def test_chart_full_with_symbol_only(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_full`` sends only ``symbol`` when both dates are left out."""
    fixture_server.route("/historical-price-eod/full", load_fixture("forex_chart_full.json"))
    rows = client.forex.chart_full("EURUSD")

    assert fixture_server.requests[0].target == "/historical-price-eod/full?symbol=EURUSD"
    assert len(rows) == 1


def test_chart_one_minute_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_one_minute`` maps to ``/historical-chart/1min`` and decodes a datetime row."""
    fixture_server.route("/historical-chart/1min", load_fixture("forex_chart_one_minute.json"))
    rows = client.forex.chart_one_minute("EURUSD", from_=INTRADAY_FROM, to=INTRADAY_TO)

    assert fixture_server.requests[0].target == f"/historical-chart/1min?{INTRADAY_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartIntradayBar)
    assert row.date == datetime.datetime(2026, 7, 30, 13, 17)
    assert row.open == pytest.approx(1.15203)
    assert row.close == pytest.approx(1.15204)
    assert row.volume == 76


def test_chart_five_minutes_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_five_minutes`` maps to ``/historical-chart/5min``."""
    fixture_server.route("/historical-chart/5min", load_fixture("forex_chart_five_minutes.json"))
    rows = client.forex.chart_five_minutes("EURUSD", from_=INTRADAY_FROM, to=INTRADAY_TO)

    assert fixture_server.requests[0].target == f"/historical-chart/5min?{INTRADAY_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartIntradayBar)
    assert row.date == datetime.datetime(2026, 7, 30, 13, 15)
    assert row.high == pytest.approx(1.1521)
    assert row.low == pytest.approx(1.15194)
    assert row.volume == 91


def test_chart_one_hour_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_one_hour`` maps to ``/historical-chart/1hour``."""
    fixture_server.route("/historical-chart/1hour", load_fixture("forex_chart_one_hour.json"))
    rows = client.forex.chart_one_hour("EURUSD", from_=INTRADAY_FROM, to=INTRADAY_TO)

    assert fixture_server.requests[0].target == f"/historical-chart/1hour?{INTRADAY_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartIntradayBar)
    assert row.date == datetime.datetime(2026, 7, 30, 13, 0)
    assert row.open == pytest.approx(1.1529)
    assert row.high == pytest.approx(1.15327)
    assert row.volume == 1_420


def test_chart_one_hour_omits_from_independently(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_one_hour`` sends only ``to`` when ``from_`` is left out."""
    fixture_server.route("/historical-chart/1hour", load_fixture("forex_chart_one_hour.json"))
    rows = client.forex.chart_one_hour("EURUSD", to="2024-03-01")

    assert fixture_server.requests[0].target == "/historical-chart/1hour?symbol=EURUSD&to=2024-03-01"
    assert len(rows) == 1


def test_from_keyword_is_renamed_but_the_wire_key_is_not(client: Any, fixture_server: FixtureServer) -> None:
    """The Python keyword is ``from_``; ``from`` is rejected and the wire key stays ``from``."""
    fixture_server.route("/historical-chart/5min", load_fixture("forex_chart_five_minutes.json"))
    with pytest.raises(TypeError, match="unexpected keyword argument 'from'"):
        client.forex.chart_five_minutes("EURUSD", **{"from": "2024-01-01"})
    assert fixture_server.requests == []

    client.forex.chart_five_minutes("EURUSD", from_="2024-01-01")
    assert fixture_server.requests[0].query == {"symbol": ["EURUSD"], "from": ["2024-01-01"]}
    assert "from_" not in fixture_server.requests[0].raw_query


def test_chart_dates_are_keyword_only(client: Any, fixture_server: FixtureServer) -> None:
    """A positional date after the symbol is a ``TypeError``, never a silent ``from``."""
    with pytest.raises(TypeError):
        client.forex.chart_light("EURUSD", "2026-01-27")
    assert fixture_server.requests == []
