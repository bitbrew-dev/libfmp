"""Runtime contract of ``client.market`` for the market-mover methods and the negatives.

The three mover methods take no arguments and map to the exact query-less
paths the Rust ``market_mover_endpoints.rs`` test pins. The negative cases
cover every argument kind the domain has (``date``, ``exchange_code``,
``sector``, ``industry``), the ``from`` -> ``from_`` rename, the keyword-only
filters, and the structured status and decode failures.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.market import MarketMover

SNAPSHOT_DATE = datetime.date(2024, 2, 1)


@pytest.mark.parametrize(
    ("method", "path", "fixture", "symbol", "name", "price", "change", "changes_percentage", "exchange"),
    [
        pytest.param(
            "biggest_gainers",
            "/biggest-gainers",
            "biggest_gainers.json",
            "MOTS",
            "Motus GI Holdings, Inc.",
            0.0002,
            0.0001,
            100.0,
            "OTC",
            id="gainers",
        ),
        pytest.param(
            "biggest_losers",
            "/biggest-losers",
            "biggest_losers.json",
            "SPEC",
            "Spectaire Holdings Inc.",
            0.0002,
            -0.002,
            -90.90909,
            "OTC",
            id="losers",
        ),
        pytest.param(
            "most_actives",
            "/most-actives",
            "most_actives.json",
            "LUCY",
            "Innovative Eyewear, Inc.",
            1.85,
            0.06,
            3.35196,
            "NASDAQ",
            id="actives",
        ),
    ],
)
def test_movers_take_no_arguments(
    client: Any,
    fixture_server: FixtureServer,
    method: str,
    path: str,
    fixture: str,
    symbol: str,
    name: str,
    price: float,
    change: float,
    changes_percentage: float,
    exchange: str,
) -> None:
    """Each mover method hits its own path with no query and decodes the ranked row."""
    fixture_server.route(path, load_fixture(fixture))
    rows = getattr(client.market, method)()

    assert fixture_server.requests[0].target == path
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, MarketMover)
    assert row.symbol == symbol
    assert row.name == name
    assert row.price == pytest.approx(price)
    assert row.change == pytest.approx(change)
    assert row.changes_percentage == pytest.approx(changes_percentage)
    assert row.exchange == exchange


@pytest.mark.parametrize("method", ["biggest_gainers", "biggest_losers", "most_actives"])
def test_movers_reject_arguments(client: Any, fixture_server: FixtureServer, method: str) -> None:
    """A query-less mover method called with an argument is a ``TypeError`` before any request."""
    with pytest.raises(TypeError):
        getattr(client.market, method)("NASDAQ")
    assert fixture_server.requests == []


def test_from_keyword_is_renamed_but_the_wire_key_is_not(client: Any, fixture_server: FixtureServer) -> None:
    """The Python keyword is ``from_``; ``from`` is rejected and the wire key stays ``from``."""
    fixture_server.route("/historical-sector-pe", load_fixture("historical_sector_pe.json"))
    with pytest.raises(TypeError, match="unexpected keyword argument 'from'"):
        client.market.historical_sector_pe("Energy", **{"from": "2024-02-01"})
    assert fixture_server.requests == []

    client.market.historical_sector_pe("Energy", from_="2024-02-01")
    assert fixture_server.requests[0].query == {"from": ["2024-02-01"], "sector": ["Energy"]}
    assert "from_" not in fixture_server.requests[0].raw_query


@pytest.mark.parametrize(
    ("method", "argument"),
    [
        ("sector_performance_snapshot", SNAPSHOT_DATE),
        ("industry_pe_snapshot", "2024-02-01"),
        ("historical_sector_performance", "Energy"),
        ("historical_industry_pe", "Biotechnology"),
    ],
)
def test_filters_are_keyword_only(client: Any, fixture_server: FixtureServer, method: str, argument: Any) -> None:
    """A second positional argument is a ``TypeError``, never a silent ``exchange`` or ``from``."""
    with pytest.raises(TypeError):
        getattr(client.market, method)(argument, "NASDAQ")
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("method", "value"),
    [
        ("sector_performance_snapshot", "02/01/2024"),
        ("industry_performance_snapshot", "2024-13-01"),
        ("sector_pe_snapshot", ""),
        ("industry_pe_snapshot", "2024-2-1"),
    ],
)
def test_non_iso_snapshot_date_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, value: str
) -> None:
    """A snapshot ``date`` string that is not ``YYYY-MM-DD`` fails locally, prefixed with ``date``."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.market, method)(value)
    error = raised.value
    assert str(error) == "date: value must be a valid YYYY-MM-DD date"
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("method", "required", "keyword", "value"),
    [
        ("historical_sector_performance", "Energy", "from_", "02/01/2024"),
        ("historical_sector_pe", "Energy", "to", "2024-13-01"),
        ("historical_industry_performance", "Biotechnology", "from_", ""),
        ("historical_industry_pe", "Biotechnology", "to", "2024-3-1"),
    ],
)
def test_non_iso_history_date_names_the_keyword(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    required: str,
    keyword: str,
    value: str,
) -> None:
    """A history date string that is not ``YYYY-MM-DD`` fails locally, prefixed with the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.market, method)(required, **{keyword: value})
    error = raised.value
    assert str(error) == f"{keyword}: value must be a valid YYYY-MM-DD date"
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("method", "required"),
    [
        ("sector_performance_snapshot", SNAPSHOT_DATE),
        ("industry_pe_snapshot", SNAPSHOT_DATE),
        ("historical_sector_pe", "Energy"),
        ("historical_industry_performance", "Biotechnology"),
    ],
)
def test_blank_exchange_names_the_keyword(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, required: Any
) -> None:
    """A whitespace-only ``exchange`` filter is rejected locally with the keyword as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.market, method)(required, exchange="   ")
    error = raised.value
    assert str(error) == "exchange: value must not be empty or whitespace-only"
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["historical_sector_performance", "historical_sector_pe"])
def test_blank_required_sector_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str
) -> None:
    """A whitespace-only required ``sector`` is rejected locally with the argument name as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.market, method)("  ")
    error = raised.value
    assert str(error) == "sector: value must not be empty or whitespace-only"
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["historical_industry_performance", "historical_industry_pe"])
def test_blank_required_industry_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str
) -> None:
    """A whitespace-only required ``industry`` is rejected locally with the argument name as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.market, method)(" \t ")
    error = raised.value
    assert str(error) == "industry: value must not be empty or whitespace-only"
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("method", "keyword"),
    [
        ("sector_performance_snapshot", "sector"),
        ("sector_pe_snapshot", "sector"),
        ("industry_performance_snapshot", "industry"),
        ("industry_pe_snapshot", "industry"),
    ],
)
def test_blank_snapshot_filter_names_the_keyword(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, keyword: str
) -> None:
    """A whitespace-only optional ``sector`` or ``industry`` filter is rejected with the keyword as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.market, method)(SNAPSHOT_DATE, **{keyword: ""})
    error = raised.value
    assert str(error) == f"{keyword}: value must not be empty or whitespace-only"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_status_error_names_the_endpoint(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-success status carries the libfmp endpoint id, status, and body."""
    fixture_server.route("/biggest-gainers", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.market.biggest_gainers()
    error = raised.value
    assert error.endpoint == "biggest-gainers"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_decode_error_names_the_snapshot_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-array body on a snapshot route surfaces as a decode error with the endpoint id."""
    fixture_server.route("/sector-pe-snapshot", {})
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.market.sector_pe_snapshot(SNAPSHOT_DATE)
    assert raised.value.endpoint == "sector-pe-snapshot"
