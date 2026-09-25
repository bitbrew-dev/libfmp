"""Runtime contract of ``client.commitment_of_traders`` for the three COT methods.

One test per argument shape routes the documented fixture body, calls the
method, and asserts the exact request target plus a few typed fields
(including the ``datetime.datetime`` ones). The expected targets are the ones
the Rust ``cot_endpoints.rs`` tests pin, including the provider COT symbol
``VX / Index`` whose space and slash must reach the wire form-encoded. The
negatives cover both argument kinds the domain has (``ticker`` and ``date``),
the keyword-only optionals, the query-less ``report_list`` rejecting
arguments, and the structured status and decode failures.
"""

import copy
import datetime
import pickle
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FIXTURES_DIR, FixtureServer, load_fixture
from fmp.commitment_of_traders import (
    CommitmentOfTradersNamespace,
    CotAnalysis,
    CotReport,
    CotReportListing,
)

SYMBOL = "VX / Index"
FROM = datetime.date(2024, 1, 1)
TO = datetime.date(2024, 3, 1)
FULL_QUERY = "symbol=VX+%2F+Index&from=2024-01-01&to=2024-03-01"


def test_commitment_of_traders_namespace_is_the_generated_type(client: Any) -> None:
    """``client.commitment_of_traders`` is the generated flat namespace class."""
    assert isinstance(client.commitment_of_traders, CommitmentOfTradersNamespace)


def test_report_with_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``report`` maps to ``/commitment-of-traders-report`` bare and decodes the documented row."""
    fixture_server.route("/commitment-of-traders-report", load_fixture("cot_report.json"))
    rows = client.commitment_of_traders.report()

    assert fixture_server.requests[0].target == "/commitment-of-traders-report"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CotReport)
    assert row.symbol == "VX"
    assert row.date == datetime.datetime(2024, 2, 27)
    assert row.name == "CBOE VIX (VX)"
    assert row.sector == "INDICES"
    assert row.market_and_exchange_names == "VIX FUTURES - CBOE FUTURES EXCHANGE"
    assert row.open_interest_all == 361_331
    assert row.change_in_noncomm_spread_all == 9_257
    assert row.traders_noncomm_spread_old == 101
    assert row.pct_of_oi_noncomm_long_all == pytest.approx(20.6)
    assert row.contract_units == "($1000 X INDEX)"


def test_report_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``report`` encodes ``symbol``, ``from``, ``to`` in that order and form-encodes the COT symbol."""
    fixture_server.route("/commitment-of-traders-report", load_fixture("cot_report.json"))
    rows = client.commitment_of_traders.report(symbol=SYMBOL, from_=FROM, to=TO)

    request = fixture_server.requests[0]
    assert request.target == f"/commitment-of-traders-report?{FULL_QUERY}"
    assert request.query == {"symbol": [SYMBOL], "from": ["2024-01-01"], "to": ["2024-03-01"]}
    assert len(rows) == 1
    assert isinstance(rows[0].date, datetime.datetime)


@pytest.mark.parametrize(
    ("keywords", "target"),
    [
        ({"symbol": "VX"}, "/commitment-of-traders-report?symbol=VX"),
        ({"from_": "2024-01-01"}, "/commitment-of-traders-report?from=2024-01-01"),
        ({"to": "2024-03-01"}, "/commitment-of-traders-report?to=2024-03-01"),
        ({"symbol": "VX", "to": "2024-03-01"}, "/commitment-of-traders-report?symbol=VX&to=2024-03-01"),
        ({"from_": "2024-01-01", "to": "2024-03-01"}, "/commitment-of-traders-report?from=2024-01-01&to=2024-03-01"),
    ],
)
def test_report_filters_are_independent(
    client: Any, fixture_server: FixtureServer, keywords: dict[str, str], target: str
) -> None:
    """Every report filter combination omits exactly the filters that were left out."""
    fixture_server.route("/commitment-of-traders-report", load_fixture("cot_report.json"))
    rows = client.commitment_of_traders.report(**keywords)

    assert fixture_server.requests[0].target == target
    assert len(rows) == 1


def test_analysis_with_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``analysis`` maps to ``/commitment-of-traders-analysis`` bare and decodes the documented row."""
    fixture_server.route("/commitment-of-traders-analysis", load_fixture("cot_analysis.json"))
    rows = client.commitment_of_traders.analysis()

    assert fixture_server.requests[0].target == "/commitment-of-traders-analysis"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CotAnalysis)
    assert row.symbol == "PA"
    assert row.date == datetime.datetime(2024, 2, 27)
    assert row.name == "Palladium (PA)"
    assert row.sector == "METALS"
    assert row.exchange == "PALLADIUM - NEW YORK MERCANTILE EXCHANGE"
    assert row.net_position == -12_315
    assert row.previous_net_position == -12_453
    assert row.change_in_net_position == pytest.approx(1.11)
    assert row.market_sentiment == "Increasing Bullish"
    assert row.reversal_trend is True


def test_analysis_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``analysis`` encodes ``symbol``, ``from``, ``to`` in that order and accepts ``datetime.date`` values."""
    fixture_server.route("/commitment-of-traders-analysis", load_fixture("cot_analysis.json"))
    rows = client.commitment_of_traders.analysis(symbol=SYMBOL, from_=FROM, to=TO)

    assert fixture_server.requests[0].target == f"/commitment-of-traders-analysis?{FULL_QUERY}"
    assert len(rows) == 1
    assert isinstance(rows[0].date, datetime.datetime)


@pytest.mark.parametrize(
    ("keywords", "target"),
    [
        ({"symbol": "PA"}, "/commitment-of-traders-analysis?symbol=PA"),
        ({"from_": "2020-01-01"}, "/commitment-of-traders-analysis?from=2020-01-01"),
        ({"to": "2024-01-01"}, "/commitment-of-traders-analysis?to=2024-01-01"),
        ({"from_": "2020-01-01", "to": "2024-01-01"}, "/commitment-of-traders-analysis?from=2020-01-01&to=2024-01-01"),
    ],
)
def test_analysis_filters_are_independent(
    client: Any, fixture_server: FixtureServer, keywords: dict[str, str], target: str
) -> None:
    """Every analysis filter combination omits exactly the filters that were left out."""
    fixture_server.route("/commitment-of-traders-analysis", load_fixture("cot_analysis.json"))
    rows = client.commitment_of_traders.analysis(**keywords)

    assert fixture_server.requests[0].target == target
    assert len(rows) == 1


def test_report_list_takes_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``report_list`` maps to ``/commitment-of-traders-list`` with no query string."""
    fixture_server.route("/commitment-of-traders-list", load_fixture("cot_report_list.json"))
    rows = client.commitment_of_traders.report_list()

    assert fixture_server.requests[0].target == "/commitment-of-traders-list"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CotReportListing)
    assert row.symbol == "NG"
    assert row.name == "Natural Gas (NG)"


def test_report_list_rejects_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """The query-less ``report_list`` called with a symbol is a ``TypeError`` before any request."""
    with pytest.raises(TypeError):
        client.commitment_of_traders.report_list("NG")
    assert fixture_server.requests == []


def test_from_keyword_is_renamed_but_the_wire_key_is_not(client: Any, fixture_server: FixtureServer) -> None:
    """The Python keyword is ``from_``; ``from`` is rejected and the wire key stays ``from``."""
    fixture_server.route("/commitment-of-traders-report", load_fixture("cot_report.json"))
    with pytest.raises(TypeError, match="unexpected keyword argument 'from'"):
        client.commitment_of_traders.report(**{"from": "2024-01-01"})
    assert fixture_server.requests == []

    client.commitment_of_traders.report(from_="2024-01-01")
    assert fixture_server.requests[0].query == {"from": ["2024-01-01"]}
    assert "from_" not in fixture_server.requests[0].raw_query


@pytest.mark.parametrize("method", ["report", "analysis"])
def test_optionals_are_keyword_only(client: Any, fixture_server: FixtureServer, method: str) -> None:
    """A positional symbol is a ``TypeError``, never a silent ``symbol`` filter."""
    with pytest.raises(TypeError):
        getattr(client.commitment_of_traders, method)("VX")
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["report", "analysis"])
@pytest.mark.parametrize(
    ("value", "message"),
    [
        pytest.param("", "symbol: value must not be empty or whitespace-only", id="empty"),
        pytest.param("   ", "symbol: value must not be empty or whitespace-only", id="whitespace"),
        pytest.param("VX,PA", "symbol: ticker must not contain a comma", id="comma"),
        pytest.param("VX\n/ Index", "symbol: value must not contain control characters", id="control-char"),
    ],
)
def test_invalid_symbol_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, value: str, message: str
) -> None:
    """A blank, comma, or control-character ``symbol`` is rejected locally, prefixed with the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.commitment_of_traders, method)(symbol=value)
    error = raised.value
    assert str(error) == message
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["report", "analysis"])
@pytest.mark.parametrize(
    ("keyword", "value"),
    [("from_", "01/01/2024"), ("to", "2024-13-01"), ("from_", ""), ("to", "2024-3-1")],
)
def test_non_iso_date_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, keyword: str, value: str
) -> None:
    """A date string that is not ``YYYY-MM-DD`` fails locally, prefixed with the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.commitment_of_traders, method)(symbol="VX", **{keyword: value})
    error = raised.value
    assert str(error) == f"{keyword}: value must be a valid YYYY-MM-DD date"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_status_error_names_the_report_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status carries the libfmp endpoint id, status, and body."""
    fixture_server.route("/commitment-of-traders-report", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.commitment_of_traders.report(symbol="VX")
    error = raised.value
    assert error.endpoint == "commitment-of-traders-report"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_status_error_names_the_list_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A failing list route reports the ``commitment-of-traders-list`` endpoint id and the raw body."""
    fixture_server.route("/commitment-of-traders-list", b"not-json", status=500, content_type="text/plain")
    with pytest.raises(errors.FmpStatusError) as raised:
        client.commitment_of_traders.report_list()
    assert raised.value.endpoint == "commitment-of-traders-list"
    assert raised.value.status == 500
    assert raised.value.body == "not-json"


def test_decode_error_names_the_analysis_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-array body on the analysis route surfaces as a decode error with the endpoint id."""
    fixture_server.route("/commitment-of-traders-analysis", {})
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.commitment_of_traders.analysis()
    assert raised.value.endpoint == "commitment-of-traders-analysis"


def test_decode_error_names_the_report_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A well-formed but wrongly shaped row on the report route is a decode error."""
    fixture_server.route("/commitment-of-traders-report", [{"symbol": "VX"}])
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.commitment_of_traders.report()
    assert raised.value.endpoint == "commitment-of-traders-report"


@pytest.mark.parametrize(
    ("literal", "value"),
    [("100.00", 100.0), ("1.0E2", 100.0), ("-0", 0), ("250", 250)],
    ids=["trailing-zeros", "exponent", "negative-zero", "integer"],
)
def test_decoded_number_spelling_survives_round_trips(
    client: Any, fixture_server: FixtureServer, literal: str, value: float
) -> None:
    """A provider number spelled non-canonically still compares equal after pickle, copy, and ``to_dict``."""
    raw = (FIXTURES_DIR / "cot_report.json").read_text(encoding="utf-8")
    body = raw.replace('"pctOfOpenInterestAll": 100,', f'"pctOfOpenInterestAll": {literal},')
    assert body != raw
    fixture_server.route("/commitment-of-traders-report", body.encode("utf-8"))
    row = client.commitment_of_traders.report()[0]

    assert row.pct_of_open_interest_all == value
    assert type(row.pct_of_open_interest_all) is type(value)
    assert pickle.loads(pickle.dumps(row)) == row
    assert copy.deepcopy(row) == row
    assert CotReport(**row.to_dict()) == row
