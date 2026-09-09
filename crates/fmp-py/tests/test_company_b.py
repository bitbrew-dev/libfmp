"""Runtime contract of ``client.company``: share float, M&A, governance, negatives.

The expected targets are the ones the Rust ``company_market_data_endpoints.rs``,
``company_mergers_acquisitions_endpoints.rs``, and
``company_governance_endpoints.rs`` tests pin. ``executive_compensation``
reaches the provider as ``/governance-executive-compensation``, so its status
error carries that endpoint id. The negatives cover one invalid value per
argument-kind family the domain uses: ``ticker``, ``ticker_list``, ``cik``,
``search_term``, ``benchmark_year``, ``page``, ``limit``, and ``date``.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.company import (
    AllSharesFloatRecord,
    CompanyExecutive,
    CompanyShareFloat,
    ExecutiveCompensation,
    ExecutiveCompensationBenchmark,
    MergerAcquisition,
)

SEARCH_NAME = "  Apple, Inc. / Class A  "
SEARCH_NAME_ENCODED = "++Apple%2C+Inc.+%2F+Class+A++"
BENCHMARK_YEAR = "FY 2024/25"
BENCHMARK_YEAR_ENCODED = "FY+2024%2F25"


def test_shares_float_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``shares_float`` form-encodes the ticker and decodes the datetime-stamped row."""
    fixture_server.route("/shares-float", load_fixture("company_shares_float.json"))
    rows = client.company.shares_float("AAPL / USD")

    assert fixture_server.requests[0].target == "/shares-float?symbol=AAPL+%2F+USD"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CompanyShareFloat)
    assert row.symbol == "AAPL"
    assert row.date == datetime.datetime(2026, 7, 30, 15, 48)
    assert row.free_float == pytest.approx(99.83000000136171)
    assert row.float_shares == 14_662_387_495
    assert row.outstanding_shares == 14_687_356_000
    assert row.source.startswith("https://www.sec.gov/")


def test_shares_float_all_with_page_and_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``shares_float_all`` encodes ``page`` then ``limit`` and decodes the source-less record."""
    fixture_server.route("/shares-float-all", load_fixture("company_shares_float_all.json"))
    rows = client.company.shares_float_all(page=0, limit=5001)

    assert fixture_server.requests[0].target == "/shares-float-all?page=0&limit=5001"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, AllSharesFloatRecord)
    assert row.symbol == "000001.SZ"
    assert row.date == datetime.datetime(2026, 7, 29, 14, 23, 30)
    assert row.free_float == pytest.approx(41.40900000201062)
    assert row.float_shares == 8_035_796_667
    assert row.outstanding_shares == 19_405_918_198
    assert not hasattr(row, "source")


def test_shares_float_all_without_options_sends_the_bare_path(client: Any, fixture_server: FixtureServer) -> None:
    """``shares_float_all`` with nothing set requests the bare path."""
    fixture_server.route("/shares-float-all", load_fixture("company_shares_float_all.json"))
    rows = client.company.shares_float_all()

    assert fixture_server.requests[0].target == "/shares-float-all"
    assert len(rows) == 1


def test_mergers_acquisitions_latest_with_page_and_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``mergers_acquisitions_latest`` encodes ``page`` then ``limit`` and decodes both timestamps."""
    fixture_server.route("/mergers-acquisitions-latest", load_fixture("company_mergers_acquisitions_latest.json"))
    rows = client.company.mergers_acquisitions_latest(page=0, limit=1001)

    assert fixture_server.requests[0].target == "/mergers-acquisitions-latest?page=0&limit=1001"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, MergerAcquisition)
    assert row.symbol == "AGH"
    assert row.company_name == "Aureus Greenway Holdings Inc"
    assert row.cik == "0002009312"
    assert row.targeted_symbol == "PUSA"
    assert row.transaction_date == datetime.date(2026, 7, 29)
    assert row.accepted_date == datetime.datetime(2026, 7, 29, 16, 0, 46)


def test_mergers_acquisitions_latest_with_page_only_decodes_multiple_rows(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``mergers_acquisitions_latest`` sends ``page=0`` alone and keeps row order."""
    fixture_server.route("/mergers-acquisitions-latest", load_fixture("company_mergers_acquisitions_multiple.json"))
    rows = client.company.mergers_acquisitions_latest(page=0)

    assert fixture_server.requests[0].target == "/mergers-acquisitions-latest?page=0"
    assert len(rows) == 2
    assert [row.symbol for row in rows] == ["AGH", "PEGY"]
    assert rows[1].transaction_date == datetime.date(2021, 11, 12)


def test_mergers_acquisitions_search_preserves_the_name_verbatim(client: Any, fixture_server: FixtureServer) -> None:
    """``mergers_acquisitions_search`` sends the padded, punctuated name form-encoded as ``name``."""
    fixture_server.route("/mergers-acquisitions-search", load_fixture("company_mergers_acquisitions_search.json"))
    rows = client.company.mergers_acquisitions_search(SEARCH_NAME)

    assert fixture_server.requests[0].target == f"/mergers-acquisitions-search?name={SEARCH_NAME_ENCODED}"
    assert fixture_server.requests[0].query["name"] == [SEARCH_NAME]
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, MergerAcquisition)
    assert row.symbol == "PEGY"
    assert row.targeted_company_name == "Communications Systems, Inc."
    assert row.targeted_symbol == "JCS"
    assert row.transaction_date == datetime.date(2021, 11, 12)
    assert row.accepted_date == datetime.datetime(2021, 11, 12, 9, 54, 22)


def test_key_executives_takes_one_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``key_executives`` maps to ``/key-executives`` and decodes the null-valued dynamic fields."""
    fixture_server.route("/key-executives", load_fixture("company_key_executives.json"))
    rows = client.company.key_executives("BRK.B")

    assert fixture_server.requests[0].target == "/key-executives?symbol=BRK.B"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CompanyExecutive)
    assert row.name == "Kristin Huguet Quayle"
    assert row.title == "Vice President of Worldwide Communications"
    assert row.currency_pay == "USD"
    assert row.active is True
    assert row.pay is None
    assert row.year_born is None
    assert row.title_since is None


def test_key_executives_surfaces_dynamic_json_values(client: Any, fixture_server: FixtureServer) -> None:
    """``pay``, ``year_born``, and ``title_since`` pass provider JSON through unchanged."""
    fixture_server.route("/key-executives", load_fixture("company_key_executives_dynamic.json"))
    rows = client.company.key_executives("AAPL")

    assert fixture_server.requests[0].target == "/key-executives?symbol=AAPL"
    assert len(rows) == 2
    probe = rows[1]
    assert probe.name == "Representation Probe"
    assert probe.active is False
    assert probe.pay == {"amount": "00123.450", "components": [1, True, None]}
    assert probe.title_since == 1_704_067_200
    assert probe.year_born == "01980"


def test_executive_compensation_maps_to_the_governance_path(client: Any, fixture_server: FixtureServer) -> None:
    """``executive_compensation`` reaches ``/governance-executive-compensation`` with the symbol."""
    fixture_server.route("/governance-executive-compensation", load_fixture("company_executive_compensation.json"))
    rows = client.company.executive_compensation("AAPL")

    assert fixture_server.requests[0].target == "/governance-executive-compensation?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, ExecutiveCompensation)
    assert row.symbol == "AAPL"
    assert row.cik == "0000320193"
    assert row.year == 2025
    assert row.salary == 819_231
    assert row.stock_award == 13_003_031
    assert row.total == 15_482_928
    assert row.filing_date == datetime.date(2026, 1, 8)
    assert row.accepted_date == datetime.datetime(2026, 1, 8, 16, 31, 36)


def test_executive_compensation_decodes_large_amounts(client: Any, fixture_server: FixtureServer) -> None:
    """Amounts above ``2**32`` decode as exact Python integers."""
    fixture_server.route(
        "/governance-executive-compensation", load_fixture("company_executive_compensation_large.json")
    )
    rows = client.company.executive_compensation("AAPL")

    assert len(rows) == 2
    assert rows[1].company_name == "Large Amount Probe"
    assert rows[1].salary == 5_000_000_000
    assert rows[1].all_other_compensation == 10_000_000_000
    assert rows[1].accepted_date == datetime.datetime(2026, 1, 1, 0, 0, 0)


def test_executive_compensation_benchmark_with_a_free_text_year(client: Any, fixture_server: FixtureServer) -> None:
    """``year`` is an open provider string, form-encoded verbatim."""
    fixture_server.route(
        "/executive-compensation-benchmark", load_fixture("company_executive_compensation_benchmark.json")
    )
    rows = client.company.executive_compensation_benchmark(year=BENCHMARK_YEAR)

    assert fixture_server.requests[0].target == f"/executive-compensation-benchmark?year={BENCHMARK_YEAR_ENCODED}"
    assert fixture_server.requests[0].query["year"] == [BENCHMARK_YEAR]
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, ExecutiveCompensationBenchmark)
    assert row.industry_title == "ABRASIVE, ASBESTOS & MISC NONMETALLIC MINERAL PRODS"
    assert row.year == 2024
    assert row.average_compensation == pytest.approx(784407.5555555555)


def test_executive_compensation_benchmark_without_a_year_sends_the_bare_path(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``executive_compensation_benchmark`` with no ``year`` requests the bare path."""
    fixture_server.route(
        "/executive-compensation-benchmark", load_fixture("company_executive_compensation_benchmark.json")
    )
    rows = client.company.executive_compensation_benchmark()

    assert fixture_server.requests[0].target == "/executive-compensation-benchmark"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1
    with pytest.raises(TypeError):
        client.company.executive_compensation_benchmark("2024")


@pytest.mark.parametrize(
    ("method", "keyword", "value", "message"),
    [
        pytest.param("profile", "symbol", "   ", "symbol: value must not be empty or whitespace-only", id="ticker"),
        pytest.param(
            "market_capitalization_batch",
            "symbols",
            [],
            "symbols: ticker list must contain at least one ticker",
            id="ticker-list",
        ),
        pytest.param("profile_by_cik", "cik", "", "cik: ", id="cik"),
        pytest.param("mergers_acquisitions_search", "name", " ", "name: ", id="search-term"),
        pytest.param("executive_compensation_benchmark", "year", "", "year: ", id="benchmark-year"),
        pytest.param("delisted_companies", "page", -1, "page: must be an integer from 0 through 4294967295", id="page"),
        pytest.param("shares_float_all", "limit", -1, "limit: ", id="limit"),
        pytest.param(
            "historical_market_capitalization",
            "to",
            "16/04/2026",
            "to: value must be a valid YYYY-MM-DD date",
            id="date",
        ),
    ],
)
def test_invalid_values_name_the_argument(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    keyword: str,
    value: Any,
    message: str,
) -> None:
    """Each kind family fails locally with ``FmpValidationError`` prefixed by the keyword."""
    kwargs: dict[str, Any] = {keyword: value}
    if method == "historical_market_capitalization":
        kwargs = {"symbol": "AAPL", keyword: value}
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.company, method)(**kwargs)
    error = raised.value
    assert str(error).startswith(message)
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_from_keyword_is_renamed_but_the_wire_key_is_not(client: Any, fixture_server: FixtureServer) -> None:
    """The Python keyword is ``from_``; ``from`` is a syntax-level keyword and the wire key stays ``from``."""
    fixture_server.route(
        "/historical-market-capitalization", load_fixture("company_historical_market_capitalization.json")
    )
    with pytest.raises(TypeError):
        client.company.historical_market_capitalization("AAPL", **{"from": "2026-04-16"})
    client.company.historical_market_capitalization("AAPL", from_="2026-04-16")
    assert fixture_server.requests[0].query == {"symbol": ["AAPL"], "from": ["2026-04-16"]}
    assert "from_" not in fixture_server.requests[0].raw_query


def test_status_error_carries_the_governance_endpoint_id(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on ``executive_compensation`` names the governance endpoint."""
    fixture_server.route("/governance-executive-compensation", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.company.executive_compensation("AAPL")
    error = raised.value
    assert error.endpoint == "governance-executive-compensation"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_decode_error_on_a_non_json_body(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-JSON 200 body maps to ``FmpDecodeError`` and names the endpoint."""
    fixture_server.route("/stock-peers", b"not-json", content_type="text/plain")
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.company.stock_peers("AAPL")
    assert raised.value.endpoint == "stock-peers"
