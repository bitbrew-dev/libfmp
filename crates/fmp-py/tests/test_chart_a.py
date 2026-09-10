"""Runtime contract of ``client.chart`` for the four end-of-day methods.

The expected targets are the ones the Rust ``chart_eod_core_endpoints.rs`` and
``chart_eod_adjusted_endpoints.rs`` tests pin: ``historical-price-eod/light``,
``full``, ``non-split-adjusted``, and ``dividend-adjusted`` with ``symbol``,
then ``from``, then ``to``, each date independently optional. Every EOD row
carries a ``datetime.date``; the models are the ones the indexes domain already
reuses from ``fmp.chart``. Floats go through ``pytest.approx`` because
serde_json's parser can differ from Python's by one ULP.
"""

import datetime
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.chart import (
    ChartNamespace,
    StockChartAdjustedBar,
    StockChartFullBar,
    StockChartLightBar,
)

EOD_FROM = datetime.date(2026, 4, 30)
EOD_TO = datetime.date(2026, 7, 30)
EOD_QUERY = "symbol=AAPL&from=2026-04-30&to=2026-07-30"
ESCAPED_QUERY = "symbol=BRK.B+%2F+Class+A&from=2026-04-30&to=2026-07-30"


def test_chart_namespace_is_the_generated_type(client: Any) -> None:
    """``client.chart`` is the generated flat namespace class."""
    assert isinstance(client.chart, ChartNamespace)


def test_light_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``light`` encodes ``symbol``, ``from``, ``to`` in that order and decodes the compact row."""
    fixture_server.route("/historical-price-eod/light", load_fixture("stock_chart_light.json"))
    rows = client.chart.light("AAPL", from_=EOD_FROM, to=EOD_TO)

    assert fixture_server.requests[0].path == "/historical-price-eod/light"
    assert fixture_server.requests[0].target == f"/historical-price-eod/light?{EOD_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartLightBar)
    assert row.symbol == "AAPL"
    assert row.date == datetime.date(2026, 7, 30)
    assert row.price == pytest.approx(332.39)
    assert row.volume == 29207295


def test_light_with_from_only(client: Any, fixture_server: FixtureServer) -> None:
    """``light`` accepts an ISO string for ``from_`` and leaves ``to`` out."""
    fixture_server.route("/historical-price-eod/light", load_fixture("stock_chart_light.json"))
    rows = client.chart.light("AAPL", from_="2026-04-30")

    assert fixture_server.requests[0].target == "/historical-price-eod/light?symbol=AAPL&from=2026-04-30"
    assert len(rows) == 1


def test_light_escapes_the_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``light`` form-encodes spaces and slashes in the symbol the way the Rust proxy test pins."""
    fixture_server.route("/historical-price-eod/light", load_fixture("stock_chart_light.json"))
    client.chart.light("BRK.B / Class A", from_=EOD_FROM, to=EOD_TO)

    assert fixture_server.requests[0].target == f"/historical-price-eod/light?{ESCAPED_QUERY}"
    assert fixture_server.requests[0].query["symbol"] == ["BRK.B / Class A"]


def test_full_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``full`` maps to ``/historical-price-eod/full`` and decodes the OHLC row."""
    fixture_server.route("/historical-price-eod/full", load_fixture("stock_chart_full.json"))
    rows = client.chart.full("AAPL", from_="2026-04-30", to="2026-07-30")

    assert fixture_server.requests[0].target == f"/historical-price-eod/full?{EOD_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartFullBar)
    assert row.symbol == "AAPL"
    assert row.date == datetime.date(2026, 7, 30)
    assert row.open == pytest.approx(333.13)
    assert row.high == pytest.approx(334.48)
    assert row.low == pytest.approx(329.59)
    assert row.close == pytest.approx(332.39)
    assert row.volume == 29207295
    assert row.change == pytest.approx(-0.74)
    assert row.change_percent == pytest.approx(-0.2221355)
    assert row.vwap == pytest.approx(332.15)


def test_full_with_to_only(client: Any, fixture_server: FixtureServer) -> None:
    """``full`` sends ``to`` alone when ``from_`` is omitted."""
    fixture_server.route("/historical-price-eod/full", load_fixture("stock_chart_full.json"))
    rows = client.chart.full("AAPL", to=EOD_TO)

    assert fixture_server.requests[0].target == "/historical-price-eod/full?symbol=AAPL&to=2026-07-30"
    assert len(rows) == 1


def test_full_without_options_sends_only_the_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``full`` omits both dates when neither is given."""
    fixture_server.route("/historical-price-eod/full", load_fixture("stock_chart_full.json"))
    rows = client.chart.full("AAPL")

    assert fixture_server.requests[0].target == "/historical-price-eod/full?symbol=AAPL"
    assert fixture_server.requests[0].query == {"symbol": ["AAPL"]}
    assert len(rows) == 1


def test_non_split_adjusted_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``non_split_adjusted`` maps to ``/historical-price-eod/non-split-adjusted`` and decodes ``adj_*``."""
    fixture_server.route(
        "/historical-price-eod/non-split-adjusted", load_fixture("stock_chart_non_split_adjusted.json")
    )
    rows = client.chart.non_split_adjusted("AAPL", from_=EOD_FROM, to=EOD_TO)

    assert fixture_server.requests[0].target == f"/historical-price-eod/non-split-adjusted?{EOD_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartAdjustedBar)
    assert row.symbol == "AAPL"
    assert row.date == datetime.date(2026, 7, 30)
    assert row.adj_open == pytest.approx(333.13)
    assert row.adj_high == pytest.approx(334.48)
    assert row.adj_low == pytest.approx(329.59)
    assert row.adj_close == pytest.approx(332.39)
    assert row.volume == 29207295


def test_non_split_adjusted_with_from_only(client: Any, fixture_server: FixtureServer) -> None:
    """``non_split_adjusted`` sends ``from`` alone, matching the Rust direct-auth test."""
    fixture_server.route(
        "/historical-price-eod/non-split-adjusted", load_fixture("stock_chart_non_split_adjusted.json")
    )
    rows = client.chart.non_split_adjusted("AAPL", from_=EOD_FROM)

    assert fixture_server.requests[0].target == "/historical-price-eod/non-split-adjusted?symbol=AAPL&from=2026-04-30"
    assert len(rows) == 1


def test_dividend_adjusted_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``dividend_adjusted`` maps to ``/historical-price-eod/dividend-adjusted`` with the shared row type."""
    fixture_server.route(
        "/historical-price-eod/dividend-adjusted", load_fixture("stock_chart_dividend_adjusted.json")
    )
    rows = client.chart.dividend_adjusted("AAPL", from_="2026-04-30", to=EOD_TO)

    assert fixture_server.requests[0].target == f"/historical-price-eod/dividend-adjusted?{EOD_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartAdjustedBar)
    assert row.date == datetime.date(2026, 7, 30)
    assert row.adj_close == pytest.approx(332.39)
    assert row.adj_low == pytest.approx(329.59)
    assert row.volume == 29207295


def test_dividend_adjusted_with_to_only(client: Any, fixture_server: FixtureServer) -> None:
    """``dividend_adjusted`` sends ``to`` alone, matching the Rust direct-auth test."""
    fixture_server.route(
        "/historical-price-eod/dividend-adjusted", load_fixture("stock_chart_dividend_adjusted.json")
    )
    rows = client.chart.dividend_adjusted("AAPL", to="2026-07-30")

    assert fixture_server.requests[0].target == "/historical-price-eod/dividend-adjusted?symbol=AAPL&to=2026-07-30"
    assert len(rows) == 1


@pytest.mark.parametrize("method", ["light", "full", "non_split_adjusted", "dividend_adjusted"])
def test_eod_methods_preserve_empty_arrays(client: Any, fixture_server: FixtureServer, method: str) -> None:
    """Each EOD method returns ``[]`` for the server's default empty array."""
    assert getattr(client.chart, method)("AAPL") == []
    assert len(fixture_server.requests) == 1
    assert fixture_server.requests[0].raw_query == "symbol=AAPL"


@pytest.mark.parametrize("method", ["light", "full", "non_split_adjusted", "dividend_adjusted"])
def test_eod_dates_are_keyword_only(client: Any, fixture_server: FixtureServer, method: str) -> None:
    """A positional date after the symbol is a ``TypeError`` before any request."""
    with pytest.raises(TypeError):
        getattr(client.chart, method)("AAPL", EOD_FROM)
    assert fixture_server.requests == []
