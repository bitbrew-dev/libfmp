"""Runtime contract of ``client.indexes`` for the directory, quote, and chart methods.

One test per method routes the documented fixture body, calls the method with
one argument shape, and asserts the exact request target plus a few typed
fields (including the ``datetime.date`` / ``datetime.datetime`` ones). The
expected targets are the ones the Rust ``indexes_directory_quote_endpoints.rs``
and ``indexes_history_endpoints.rs`` tests pin: the caret in ``^VIX`` is
form-encoded as ``%5E``. The constituent methods and the shared negative
cases live in ``test_indexes_b.py``.
"""

import datetime
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.chart import StockChartFullBar, StockChartIntradayBar, StockChartLightBar
from fmp.indexes import IndexesNamespace, IndexListing
from fmp.quote import Quote, QuoteShort

EOD_FROM = datetime.date(2026, 1, 27)
EOD_TO = datetime.date(2026, 4, 27)
INTRADAY_FROM = datetime.date(2024, 1, 1)
INTRADAY_TO = datetime.date(2024, 3, 1)
EOD_QUERY = "symbol=%5EVIX&from=2026-01-27&to=2026-04-27"
INTRADAY_QUERY = "symbol=%5EVIX&from=2024-01-01&to=2024-03-01"


def test_indexes_namespace_is_the_generated_type(client: Any) -> None:
    """``client.indexes`` is the generated flat namespace class."""
    assert isinstance(client.indexes, IndexesNamespace)


def test_list_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``list`` maps to ``/index-list`` with no query string."""
    fixture_server.route("/index-list", load_fixture("indexes_list.json"))
    rows = client.indexes.list()

    assert fixture_server.requests[0].target == "/index-list"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, IndexListing)
    assert row.symbol == "^TTIN"
    assert row.name == "S&P/TSX Capped Industrials Index"
    assert row.exchange == "TSX"
    assert row.currency == "CAD"


def test_quote_encodes_the_caret_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``quote`` reuses the generic ``/quote`` route and decodes the full quote row."""
    fixture_server.route("/quote", load_fixture("indexes_quote.json"))
    rows = client.indexes.quote("^VIX")

    assert fixture_server.requests[0].target == "/quote?symbol=%5EVIX"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, Quote)
    assert row.symbol == "^VIX"
    assert row.name == "CBOE Volatility Index"
    assert row.price == pytest.approx(18.1)
    assert row.change_percentage == pytest.approx(-12.39109)
    assert row.timestamp == 1_785_430_786


def test_quote_short_encodes_the_caret_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``quote_short`` reuses the generic ``/quote-short`` route."""
    fixture_server.route("/quote-short", load_fixture("indexes_quote_short.json"))
    rows = client.indexes.quote_short("^VIX")

    assert fixture_server.requests[0].target == "/quote-short?symbol=%5EVIX"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, QuoteShort)
    assert row.symbol == "^VIX"
    assert row.price == pytest.approx(18.1)
    assert row.change == pytest.approx(-2.56)
    assert row.volume == 0


def test_chart_light_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_light`` encodes ``symbol``, ``from``, ``to`` in that order and decodes a date row."""
    fixture_server.route("/historical-price-eod/light", load_fixture("indexes_chart_light.json"))
    rows = client.indexes.chart_light("^VIX", from_=EOD_FROM, to=EOD_TO)

    assert fixture_server.requests[0].target == f"/historical-price-eod/light?{EOD_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartLightBar)
    assert row.symbol == "^VIX"
    assert row.date == datetime.date(2026, 7, 30)
    assert row.price == pytest.approx(17.9)
    assert row.volume == 0


def test_chart_light_omits_to_independently(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_light`` accepts an ISO string for ``from_`` and leaves ``to`` out."""
    fixture_server.route("/historical-price-eod/light", load_fixture("indexes_chart_light.json"))
    rows = client.indexes.chart_light("^VIX", from_="2026-01-27")

    assert fixture_server.requests[0].target == "/historical-price-eod/light?symbol=%5EVIX&from=2026-01-27"
    assert len(rows) == 1


def test_chart_full_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_full`` maps to ``/historical-price-eod/full`` and decodes the OHLC row."""
    fixture_server.route("/historical-price-eod/full", load_fixture("indexes_chart_full.json"))
    rows = client.indexes.chart_full("^VIX", from_="2026-01-27", to="2026-04-27")

    assert fixture_server.requests[0].target == f"/historical-price-eod/full?{EOD_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartFullBar)
    assert row.symbol == "^VIX"
    assert row.date == datetime.date(2026, 7, 30)
    assert row.open == pytest.approx(19.56)
    assert row.close == pytest.approx(17.9)
    assert row.change == pytest.approx(-1.66)
    assert row.change_percent == pytest.approx(-8.48671)
    assert row.vwap == pytest.approx(18.63)


def test_chart_full_without_options_sends_only_the_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_full`` omits both dates when neither is given."""
    fixture_server.route("/historical-price-eod/full", load_fixture("indexes_chart_full.json"))
    rows = client.indexes.chart_full("^VIX")

    assert fixture_server.requests[0].target == "/historical-price-eod/full?symbol=%5EVIX"
    assert len(rows) == 1


def test_chart_one_minute_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_one_minute`` maps to ``/historical-chart/1min`` and decodes a ``datetime`` row."""
    fixture_server.route("/historical-chart/1min", load_fixture("indexes_chart_one_minute.json"))
    rows = client.indexes.chart_one_minute("^VIX", from_=INTRADAY_FROM, to=INTRADAY_TO)

    assert fixture_server.requests[0].target == f"/historical-chart/1min?{INTRADAY_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartIntradayBar)
    assert row.date == datetime.datetime(2026, 7, 30, 13, 17)
    assert row.open == pytest.approx(17.92)
    assert row.close == pytest.approx(17.91)
    assert row.volume == 0


def test_chart_five_minutes_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_five_minutes`` maps to ``/historical-chart/5min``."""
    fixture_server.route("/historical-chart/5min", load_fixture("indexes_chart_five_minutes.json"))
    rows = client.indexes.chart_five_minutes("^VIX", from_=INTRADAY_FROM, to=INTRADAY_TO)

    assert fixture_server.requests[0].target == f"/historical-chart/5min?{INTRADAY_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartIntradayBar)
    assert row.date == datetime.datetime(2026, 7, 30, 13, 15)
    assert row.open == pytest.approx(17.99)
    assert row.low == pytest.approx(17.96)


def test_chart_one_hour_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_one_hour`` maps to ``/historical-chart/1hour``."""
    fixture_server.route("/historical-chart/1hour", load_fixture("indexes_chart_one_hour.json"))
    rows = client.indexes.chart_one_hour("^VIX", from_=INTRADAY_FROM, to=INTRADAY_TO)

    assert fixture_server.requests[0].target == f"/historical-chart/1hour?{INTRADAY_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartIntradayBar)
    assert row.date == datetime.datetime(2026, 7, 30, 12, 30)
    assert row.open == pytest.approx(18.23)
    assert row.high == pytest.approx(18.35)


def test_chart_one_hour_omits_from_independently(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_one_hour`` sends only ``to`` when ``from_`` is left out."""
    fixture_server.route("/historical-chart/1hour", load_fixture("indexes_chart_one_hour.json"))
    rows = client.indexes.chart_one_hour("^VIX", to="2024-03-01")

    assert fixture_server.requests[0].target == "/historical-chart/1hour?symbol=%5EVIX&to=2024-03-01"
    assert len(rows) == 1


def test_from_keyword_is_renamed_but_the_wire_key_is_not(client: Any, fixture_server: FixtureServer) -> None:
    """The Python keyword is ``from_``; ``from`` is rejected and the wire key stays ``from``."""
    fixture_server.route("/historical-chart/5min", load_fixture("indexes_chart_five_minutes.json"))
    with pytest.raises(TypeError, match="unexpected keyword argument 'from'"):
        client.indexes.chart_five_minutes("^VIX", **{"from": "2024-01-01"})
    assert fixture_server.requests == []

    client.indexes.chart_five_minutes("^VIX", from_="2024-01-01")
    assert fixture_server.requests[0].query == {"symbol": ["^VIX"], "from": ["2024-01-01"]}
    assert "from_" not in fixture_server.requests[0].raw_query


def test_chart_dates_are_keyword_only(client: Any, fixture_server: FixtureServer) -> None:
    """A positional date after the symbol is a ``TypeError``, never a silent ``from``."""
    with pytest.raises(TypeError):
        client.indexes.chart_light("^VIX", "2026-01-27")
    assert fixture_server.requests == []
