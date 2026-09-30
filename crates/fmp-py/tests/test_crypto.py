"""Runtime contract of ``client.crypto`` for the catalog, quote, and chart methods.

One test per method routes the documented fixture body, calls the method with
one argument shape, and asserts the exact request target plus a few typed
fields (including the ``datetime.date`` / ``datetime.datetime`` ones). The
expected targets are the ones the Rust ``asset_catalog_quote_endpoints.rs``
and ``asset_history_endpoints.rs`` tests pin for ``BTCUSD``. The negative
cases cover every argument kind the domain has (``ticker`` and ``date``) plus
the structured status and decode failures.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.chart import StockChartFullBar, StockChartIntradayBar, StockChartLightBar
from fmp.crypto import CryptocurrencyListing, CryptoNamespace
from fmp.quote import Quote, QuoteShort

EOD_FROM = datetime.date(2026, 1, 27)
EOD_TO = datetime.date(2026, 4, 27)
INTRADAY_FROM = datetime.date(2024, 1, 1)
INTRADAY_TO = datetime.date(2024, 3, 1)
EOD_QUERY = "symbol=BTCUSD&from=2026-01-27&to=2026-04-27"
INTRADAY_QUERY = "symbol=BTCUSD&from=2024-01-01&to=2024-03-01"


def test_crypto_namespace_is_the_generated_type(client: Any) -> None:
    """``client.crypto`` is the generated flat namespace class."""
    assert isinstance(client.crypto, CryptoNamespace)


def test_list_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``list`` maps to ``/cryptocurrency-list`` with no query string and decodes the ICO date."""
    fixture_server.route("/cryptocurrency-list", load_fixture("cryptocurrency_list.json"))
    rows = client.crypto.list()

    assert fixture_server.requests[0].target == "/cryptocurrency-list"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CryptocurrencyListing)
    assert row.symbol == "MIOTAUSD"
    assert row.name == "IOTA USD"
    assert row.exchange == "CCC"
    assert row.ico_date == datetime.date(2017, 11, 9)
    assert row.circulating_supply == 4_232_705_124
    assert row.total_supply == 4_788_606_639


def test_list_decodes_integral_float_and_fractional_supplies(client: Any, fixture_server: FixtureServer) -> None:
    """Issue #339: a fractional or integral-float supply decodes as a ``float``."""
    fixture_server.route("/cryptocurrency-list", load_fixture("cryptocurrency_list_fractional_synthetic.json"))
    rows = client.crypto.list()

    assert len(rows) == 1
    assert isinstance(rows[0].circulating_supply, float)
    assert rows[0].circulating_supply == 4_232_705_124.5
    assert rows[0].total_supply == 4_788_606_639.0


def test_quote_reuses_the_shared_quote_route(client: Any, fixture_server: FixtureServer) -> None:
    """``quote`` reuses the generic ``/quote`` route and decodes the full quote row."""
    fixture_server.route("/quote", load_fixture("cryptocurrency_quote.json"))
    rows = client.crypto.quote("BTCUSD")

    assert fixture_server.requests[0].target == "/quote?symbol=BTCUSD"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, Quote)
    assert row.symbol == "BTCUSD"
    assert row.name == "Bitcoin USD"
    assert row.exchange == "CRYPTO"
    assert row.price == pytest.approx(64_756.84)
    assert row.change_percentage == pytest.approx(1.33631)
    assert row.market_cap == 1_293_361_815_015
    assert row.timestamp == 1_785_430_805


def test_quote_short_reuses_the_shared_quote_short_route(client: Any, fixture_server: FixtureServer) -> None:
    """``quote_short`` reuses the generic ``/quote-short`` route."""
    fixture_server.route("/quote-short", load_fixture("cryptocurrency_quote_short.json"))
    rows = client.crypto.quote_short("BTCUSD")

    assert fixture_server.requests[0].target == "/quote-short?symbol=BTCUSD"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, QuoteShort)
    assert row.symbol == "BTCUSD"
    assert row.price == pytest.approx(64_756.84)
    assert row.change == pytest.approx(853.94)
    assert row.volume == 32_030_003_200


def test_chart_light_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_light`` encodes ``symbol``, ``from``, ``to`` in that order and decodes a date row."""
    fixture_server.route("/historical-price-eod/light", load_fixture("crypto_chart_light.json"))
    rows = client.crypto.chart_light("BTCUSD", from_=EOD_FROM, to=EOD_TO)

    assert fixture_server.requests[0].target == f"/historical-price-eod/light?{EOD_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartLightBar)
    assert row.symbol == "BTCUSD"
    assert row.date == datetime.date(2026, 7, 30)
    assert row.price == pytest.approx(64_766.98828)
    assert row.volume == 32_030_003_200


def test_chart_light_omits_to_independently(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_light`` accepts an ISO string for ``from_`` and leaves ``to`` out."""
    fixture_server.route("/historical-price-eod/light", load_fixture("crypto_chart_light.json"))
    rows = client.crypto.chart_light("BTCUSD", from_="2026-01-27")

    assert fixture_server.requests[0].target == "/historical-price-eod/light?symbol=BTCUSD&from=2026-01-27"
    assert len(rows) == 1


def test_chart_full_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_full`` maps to ``/historical-price-eod/full`` and decodes the OHLC row."""
    fixture_server.route("/historical-price-eod/full", load_fixture("crypto_chart_full.json"))
    rows = client.crypto.chart_full("BTCUSD", from_="2026-01-27", to="2026-04-27")

    assert fixture_server.requests[0].target == f"/historical-price-eod/full?{EOD_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartFullBar)
    assert row.symbol == "BTCUSD"
    assert row.date == datetime.date(2026, 7, 30)
    assert row.open == pytest.approx(63_902.9)
    assert row.close == pytest.approx(64_766.98828)
    assert row.change == pytest.approx(864.09)
    assert row.change_percent == pytest.approx(1.35219)
    assert row.vwap == pytest.approx(64_451.05)
    assert row.volume == 32_030_003_200


def test_chart_full_without_options_sends_only_the_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_full`` omits both dates when neither is given."""
    fixture_server.route("/historical-price-eod/full", load_fixture("crypto_chart_full.json"))
    rows = client.crypto.chart_full("BTCUSD")

    assert fixture_server.requests[0].target == "/historical-price-eod/full?symbol=BTCUSD"
    assert len(rows) == 1


def test_chart_one_minute_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_one_minute`` maps to ``/historical-chart/1min`` and decodes a ``datetime`` row."""
    fixture_server.route("/historical-chart/1min", load_fixture("crypto_chart_one_minute.json"))
    rows = client.crypto.chart_one_minute("BTCUSD", from_=INTRADAY_FROM, to=INTRADAY_TO)

    assert fixture_server.requests[0].target == f"/historical-chart/1min?{INTRADAY_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartIntradayBar)
    assert row.date == datetime.datetime(2026, 7, 30, 13, 16)
    assert row.open == pytest.approx(64_727.04)
    assert row.close == pytest.approx(64_734.67)
    assert row.volume == 0


def test_chart_five_minutes_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_five_minutes`` maps to ``/historical-chart/5min``."""
    fixture_server.route("/historical-chart/5min", load_fixture("crypto_chart_five_minutes.json"))
    rows = client.crypto.chart_five_minutes("BTCUSD", from_=INTRADAY_FROM, to=INTRADAY_TO)

    assert fixture_server.requests[0].target == f"/historical-chart/5min?{INTRADAY_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartIntradayBar)
    assert row.date == datetime.datetime(2026, 7, 30, 13, 15)
    assert row.open == pytest.approx(64_723.74)
    assert row.low == pytest.approx(64_723.73828)


def test_chart_one_hour_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_one_hour`` maps to ``/historical-chart/1hour``."""
    fixture_server.route("/historical-chart/1hour", load_fixture("crypto_chart_one_hour.json"))
    rows = client.crypto.chart_one_hour("BTCUSD", from_=INTRADAY_FROM, to=INTRADAY_TO)

    assert fixture_server.requests[0].target == f"/historical-chart/1hour?{INTRADAY_QUERY}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockChartIntradayBar)
    assert row.date == datetime.datetime(2026, 7, 30, 13, 0)
    assert row.open == pytest.approx(64_760.34)
    assert row.high == pytest.approx(64_808.03)


def test_chart_one_hour_omits_from_independently(client: Any, fixture_server: FixtureServer) -> None:
    """``chart_one_hour`` sends only ``to`` when ``from_`` is left out, as the Rust direct-auth test pins."""
    fixture_server.route("/historical-chart/1hour", load_fixture("crypto_chart_one_hour.json"))
    rows = client.crypto.chart_one_hour("BTCUSD", to="2024-03-01")

    assert fixture_server.requests[0].target == "/historical-chart/1hour?symbol=BTCUSD&to=2024-03-01"
    assert len(rows) == 1


def test_from_keyword_is_renamed_but_the_wire_key_is_not(client: Any, fixture_server: FixtureServer) -> None:
    """The Python keyword is ``from_``; ``from`` is rejected and the wire key stays ``from``."""
    fixture_server.route("/historical-chart/5min", load_fixture("crypto_chart_five_minutes.json"))
    with pytest.raises(TypeError, match="unexpected keyword argument 'from'"):
        client.crypto.chart_five_minutes("BTCUSD", **{"from": "2024-01-01"})
    assert fixture_server.requests == []

    client.crypto.chart_five_minutes("BTCUSD", from_="2024-01-01")
    assert fixture_server.requests[0].query == {"symbol": ["BTCUSD"], "from": ["2024-01-01"]}
    assert "from_" not in fixture_server.requests[0].raw_query


def test_chart_dates_are_keyword_only(client: Any, fixture_server: FixtureServer) -> None:
    """A positional date after the symbol is a ``TypeError``, never a silent ``from``."""
    with pytest.raises(TypeError):
        client.crypto.chart_light("BTCUSD", "2026-01-27")
    assert fixture_server.requests == []


def test_list_rejects_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """The query-less ``list`` called with a symbol is a ``TypeError`` before any request."""
    with pytest.raises(TypeError):
        client.crypto.list("BTCUSD")
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["quote", "quote_short", "chart_light", "chart_one_minute"])
def test_blank_symbol_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str
) -> None:
    """A whitespace-only ``symbol`` is rejected locally with the argument name as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.crypto, method)("  ")
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
        client.crypto.chart_full("BTCUSD", **{keyword: value})
    error = raised.value
    assert str(error) == f"{keyword}: value must be a valid YYYY-MM-DD date"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_status_error_names_the_endpoint(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-success status carries the libfmp endpoint id, status, and body."""
    fixture_server.route("/cryptocurrency-list", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.crypto.list()
    error = raised.value
    assert error.endpoint == "cryptocurrency-list"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_decode_error_names_the_chart_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-array body on a chart route surfaces as a decode error with the shared endpoint id."""
    fixture_server.route("/historical-chart/1min", {})
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.crypto.chart_one_minute("BTCUSD")
    assert raised.value.endpoint == "historical-chart/1min"


def test_list_decodes_null_supplies_and_empty_ico_date_to_none(client: Any, fixture_server: FixtureServer) -> None:
    """Null supplies and an empty ``icoDate`` decode to ``None``."""
    body = load_fixture("cryptocurrency_list.json")
    body[0]["icoDate"] = ""
    body[0]["circulatingSupply"] = None
    body[0]["totalSupply"] = None
    fixture_server.route("/cryptocurrency-list", body)
    rows = client.crypto.list()

    assert rows[0].ico_date is None
    assert rows[0].circulating_supply is None
    assert rows[0].total_supply is None
