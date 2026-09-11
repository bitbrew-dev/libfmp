"""Runtime contract of ``client.commodities`` on the route-table fixture server.

One test per method routes the documented fixture body, calls the method with
one argument shape, and asserts the exact request target plus a few typed
fields (including the ``datetime.date`` / ``datetime.datetime`` ones). The
expected targets are the ones the Rust ``asset_catalog_quote_endpoints.rs``
and ``asset_history_endpoints.rs`` tests pin: ``list`` has its own
``commodities-list`` path while the quote and chart methods reuse the shared
routes. The negative cases cover every argument kind the domain has
(``ticker`` and ``date``) plus the structured status and decode failures.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.chart import StockChartFullBar, StockChartIntradayBar, StockChartLightBar
from fmp.commodities import CommoditiesNamespace, CommodityListing
from fmp.quote import Quote, QuoteShort

EOD_FROM = datetime.date(2026, 1, 27)
EOD_TO = datetime.date(2026, 4, 27)
INTRADAY_FROM = datetime.date(2024, 1, 1)
INTRADAY_TO = datetime.date(2024, 3, 1)
EOD_QUERY = "symbol=GCUSD&from=2026-01-27&to=2026-04-27"
INTRADAY_QUERY = "symbol=GCUSD&from=2024-01-01&to=2024-03-01"


def test_commodities_namespace_is_the_generated_type(client: Any) -> None:
    """``client.commodities`` is the generated flat namespace class."""
    assert isinstance(client.commodities, CommoditiesNamespace)


def test_list_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``list`` maps to ``/commodities-list`` with no query string and keeps a null exchange."""
    fixture_server.route("/commodities-list", load_fixture("commodities_list.json"))
    rows = client.commodities.list()

    assert fixture_server.requests[0].target == "/commodities-list"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CommodityListing)
    assert row.symbol == "ZMUSD"
    assert row.name == "Soybean Meal Futures"
    assert row.exchange is None
    assert row.trade_month == "Dec"
    assert row.currency == "USD"


def test_quote_reuses_the_shared_route(client: Any, fixture_server: FixtureServer) -> None:
    """``quote`` reuses the generic ``/quote`` route and decodes the full quote row."""
    fixture_server.route("/quote", load_fixture("commodities_quote.json"))
    rows = client.commodities.quote("GCUSD")

    assert fixture_server.requests[0].target == "/quote?symbol=GCUSD"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, Quote)
    assert row.symbol == "GCUSD"
    assert row.name == "Gold Futures"
    assert row.exchange == "COMMODITY"
    assert row.price == pytest.approx(4168.3)
    assert row.change_percentage == pytest.approx(3.27032)
    assert row.market_cap is None
    assert row.timestamp == 1_785_430_211


def test_quote_short_reuses_the_shared_route(client: Any, fixture_server: FixtureServer) -> None:
    """``quote_short`` reuses the generic ``/quote-short`` route."""
    fixture_server.route("/quote-short", load_fixture("commodities_quote_short.json"))
    rows = client.commodities.quote_short("GCUSD")

    assert fixture_server.requests[0].target == "/quote-short?symbol=GCUSD"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, QuoteShort)
    assert row.symbol == "GCUSD"
    assert row.price == pytest.approx(4168.3)
    assert row.change == pytest.approx(132.0)
    assert row.volume == 125_925


def test_chart_light_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_light`` encodes ``symbol``, ``from``, ``to`` in that order and decodes a date row."""
    fixture_server.route("/historical-price-eod/light", load_fixture("commodity_chart_light.json"))
    rows = client.commodities.chart_light("GCUSD", from_=EOD_FROM, to=EOD_TO)

    assert fixture_server.requests[0].target == f"/historical-price-eod/light?{EOD_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartLightBar)
    assert row.symbol == "GCUSD"
    assert row.date == datetime.date(2026, 7, 30)
    assert row.price == pytest.approx(4170.0)
    assert row.volume == 126_573


def test_chart_light_omits_to_independently(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_light`` accepts an ISO string for ``from_`` and leaves ``to`` out."""
    fixture_server.route("/historical-price-eod/light", load_fixture("commodity_chart_light.json"))
    rows = client.commodities.chart_light("GCUSD", from_="2026-01-27")

    assert fixture_server.requests[0].target == "/historical-price-eod/light?symbol=GCUSD&from=2026-01-27"
    assert len(rows) == 1


def test_chart_full_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_full`` maps to ``/historical-price-eod/full`` and decodes the OHLC row."""
    fixture_server.route("/historical-price-eod/full", load_fixture("commodity_chart_full.json"))
    rows = client.commodities.chart_full("GCUSD", from_="2026-01-27", to="2026-04-27")

    assert fixture_server.requests[0].target == f"/historical-price-eod/full?{EOD_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartFullBar)
    assert row.symbol == "GCUSD"
    assert row.date == datetime.date(2026, 7, 30)
    assert row.open == pytest.approx(4126.7)
    assert row.close == pytest.approx(4170.0)
    assert row.change == pytest.approx(43.3)
    assert row.change_percent == pytest.approx(1.04926)
    assert row.vwap == pytest.approx(4145.07)


def test_chart_full_without_options_sends_only_the_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_full`` omits both dates when neither is given."""
    fixture_server.route("/historical-price-eod/full", load_fixture("commodity_chart_full.json"))
    rows = client.commodities.chart_full("GCUSD")

    assert fixture_server.requests[0].target == "/historical-price-eod/full?symbol=GCUSD"
    assert len(rows) == 1


def test_chart_one_minute_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_one_minute`` maps to ``/historical-chart/1min`` and decodes a ``datetime`` row."""
    fixture_server.route("/historical-chart/1min", load_fixture("commodity_chart_one_minute.json"))
    rows = client.commodities.chart_one_minute("GCUSD", from_=INTRADAY_FROM, to=INTRADAY_TO)

    assert fixture_server.requests[0].target == f"/historical-chart/1min?{INTRADAY_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartIntradayBar)
    assert row.date == datetime.datetime(2026, 7, 30, 13, 6)
    assert row.open == pytest.approx(4167.4)
    assert row.close == pytest.approx(4166.8)
    assert row.volume == 59


def test_chart_five_minutes_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_five_minutes`` maps to ``/historical-chart/5min``."""
    fixture_server.route("/historical-chart/5min", load_fixture("commodity_chart_five_minutes.json"))
    rows = client.commodities.chart_five_minutes("GCUSD", from_=INTRADAY_FROM, to=INTRADAY_TO)

    assert fixture_server.requests[0].target == f"/historical-chart/5min?{INTRADAY_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartIntradayBar)
    assert row.date == datetime.datetime(2026, 7, 30, 13, 5)
    assert row.open == pytest.approx(4166.6)
    assert row.low == pytest.approx(4166.6)
    assert row.volume == 103


def test_chart_one_hour_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_one_hour`` maps to ``/historical-chart/1hour``."""
    fixture_server.route("/historical-chart/1hour", load_fixture("commodity_chart_one_hour.json"))
    rows = client.commodities.chart_one_hour("GCUSD", from_=INTRADAY_FROM, to=INTRADAY_TO)

    assert fixture_server.requests[0].target == f"/historical-chart/1hour?{INTRADAY_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartIntradayBar)
    assert row.date == datetime.datetime(2026, 7, 30, 13, 0)
    assert row.open == pytest.approx(4169.3)
    assert row.high == pytest.approx(4170.0)
    assert row.volume == 690


def test_chart_one_hour_omits_from_independently(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_one_hour`` sends only ``to`` when ``from_`` is left out."""
    fixture_server.route("/historical-chart/1hour", load_fixture("commodity_chart_one_hour.json"))
    rows = client.commodities.chart_one_hour("GCUSD", to="2024-03-01")

    assert fixture_server.requests[0].target == "/historical-chart/1hour?symbol=GCUSD&to=2024-03-01"
    assert len(rows) == 1


def test_from_keyword_is_renamed_but_the_wire_key_is_not(client: Any, fixture_server: FixtureServer) -> None:
    """The Python keyword is ``from_``; ``from`` is rejected and the wire key stays ``from``."""
    fixture_server.route("/historical-chart/5min", load_fixture("commodity_chart_five_minutes.json"))
    with pytest.raises(TypeError, match="unexpected keyword argument 'from'"):
        client.commodities.chart_five_minutes("GCUSD", **{"from": "2024-01-01"})
    assert fixture_server.requests == []

    client.commodities.chart_five_minutes("GCUSD", from_="2024-01-01")
    assert fixture_server.requests[0].query == {"symbol": ["GCUSD"], "from": ["2024-01-01"]}
    assert "from_" not in fixture_server.requests[0].raw_query


def test_chart_dates_are_keyword_only(client: Any, fixture_server: FixtureServer) -> None:
    """A positional date after the symbol is a ``TypeError``, never a silent ``from``."""
    with pytest.raises(TypeError):
        client.commodities.chart_light("GCUSD", "2026-01-27")
    assert fixture_server.requests == []


def test_list_rejects_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """The query-less ``list`` called with a symbol is a ``TypeError`` before any request."""
    with pytest.raises(TypeError):
        client.commodities.list("GCUSD")
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["quote", "quote_short", "chart_light", "chart_one_minute"])
def test_blank_symbol_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str
) -> None:
    """A whitespace-only ``symbol`` is rejected locally with the argument name as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.commodities, method)("  ")
    error = raised.value
    assert str(error) == "symbol: value must not be empty or whitespace-only"
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("keyword", "value"),
    [("from_", "06/03/2026"), ("to", "2026-13-01"), ("from_", ""), ("to", "2026-6-6")],
)
def test_non_iso_date_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, keyword: str, value: str
) -> None:
    """A date string that is not ``YYYY-MM-DD`` fails locally, prefixed with the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.commodities.chart_full("GCUSD", **{keyword: value})
    error = raised.value
    assert str(error) == f"{keyword}: value must be a valid YYYY-MM-DD date"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_status_error_names_the_endpoint(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-success status carries the libfmp endpoint id, status, and body."""
    fixture_server.route("/commodities-list", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.commodities.list()
    error = raised.value
    assert error.endpoint == "commodities-list"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_decode_error_names_the_chart_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-array body on a chart route surfaces as a decode error with the endpoint id."""
    fixture_server.route("/historical-price-eod/light", {})
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.commodities.chart_light("GCUSD")
    assert raised.value.endpoint == "historical-price-eod/light"
