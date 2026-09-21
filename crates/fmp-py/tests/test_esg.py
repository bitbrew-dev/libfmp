"""Runtime contract of ``client.esg`` for the three ESG methods.

One test per argument shape routes the documented fixture body, calls the
method, and asserts the exact request target plus a few typed fields
(including the ``datetime.date`` ones on the disclosure row). The expected
targets are the ones the Rust ``esg_endpoints.rs`` test pins: the two
symbol routes form-encode the ticker, and ``benchmark`` sends the bare path
unless ``year`` is set. The negatives cover both argument kinds the domain
has (``ticker`` and ``benchmark_year``), the keyword-only ``year``, and the
structured status and decode failures.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.esg import EsgBenchmark, EsgDisclosure, EsgNamespace, EsgRating

SPACED_SYMBOL = "BRK.B / Class A"
SPACED_SYMBOL_ENCODED = "BRK.B+%2F+Class+A"
BENCHMARK_YEAR = "FY 2024/25"
BENCHMARK_YEAR_ENCODED = "FY+2024%2F25"


def test_esg_namespace_is_the_generated_type(client: Any) -> None:
    """``client.esg`` is the generated flat namespace class."""
    assert isinstance(client.esg, EsgNamespace)


def test_disclosures_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``disclosures`` form-encodes the ticker and decodes the two dated fields and the scores."""
    fixture_server.route("/esg-disclosures", load_fixture("esg_disclosures.json"))
    rows = client.esg.disclosures(SPACED_SYMBOL)

    assert fixture_server.requests[0].target == f"/esg-disclosures?symbol={SPACED_SYMBOL_ENCODED}"
    assert fixture_server.requests[0].query["symbol"] == [SPACED_SYMBOL]
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EsgDisclosure)
    assert row.date == datetime.date(2026, 3, 28)
    assert row.accepted_date == datetime.date(2026, 4, 30)
    assert row.symbol == "AAPL"
    assert row.cik == "0000320193"
    assert row.company_name == "Apple Inc."
    assert row.form_type == "8-K"
    assert row.environmental_score == pytest.approx(66.29)
    assert row.social_score == pytest.approx(45.21)
    assert row.governance_score == pytest.approx(58.87)
    assert row.esg_score == pytest.approx(56.79)
    assert row.url.startswith("https://www.sec.gov/Archives/edgar/data/320193/")


def test_disclosures_with_a_plain_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``disclosures`` with an unspaced ticker sends it verbatim, as the direct-auth Rust test pins."""
    fixture_server.route("/esg-disclosures", load_fixture("esg_disclosures.json"))
    rows = client.esg.disclosures("AAPL")

    assert fixture_server.requests[0].target == "/esg-disclosures?symbol=AAPL"
    assert len(rows) == 1
    assert isinstance(rows[0].date, datetime.date)


def test_ratings_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``ratings`` form-encodes the ticker and decodes the integer fiscal year and the rating strings."""
    fixture_server.route("/esg-ratings", load_fixture("esg_ratings.json"))
    rows = client.esg.ratings(SPACED_SYMBOL)

    assert fixture_server.requests[0].target == f"/esg-ratings?symbol={SPACED_SYMBOL_ENCODED}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EsgRating)
    assert row.symbol == "AAPL"
    assert row.cik == "0000320193"
    assert row.company_name == "Apple Inc."
    assert row.industry == "CONSUMER ELECTRONICS"
    assert row.fiscal_year == 2025
    assert isinstance(row.fiscal_year, int)
    assert row.esg_risk_rating == "B"
    assert row.industry_rank == "17 out of 20"


def test_benchmark_without_a_year_sends_the_bare_path(client: Any, fixture_server: FixtureServer) -> None:
    """``benchmark`` with no ``year`` requests the bare path and decodes the sector row."""
    fixture_server.route("/esg-benchmark", load_fixture("esg_benchmark.json"))
    rows = client.esg.benchmark()

    assert fixture_server.requests[0].target == "/esg-benchmark"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EsgBenchmark)
    assert row.fiscal_year == 2023
    assert row.sector == "APPAREL RETAIL"
    assert row.environmental_score == pytest.approx(61.36)
    assert row.social_score == pytest.approx(67.44)
    assert row.governance_score == pytest.approx(68.1)
    assert row.esg_score == pytest.approx(65.63)


def test_benchmark_with_a_year(client: Any, fixture_server: FixtureServer) -> None:
    """``benchmark`` form-encodes the provider year string exactly as given."""
    fixture_server.route("/esg-benchmark", load_fixture("esg_benchmark.json"))
    rows = client.esg.benchmark(year=BENCHMARK_YEAR)

    assert fixture_server.requests[0].target == f"/esg-benchmark?year={BENCHMARK_YEAR_ENCODED}"
    assert fixture_server.requests[0].query["year"] == [BENCHMARK_YEAR]
    assert len(rows) == 1
    assert isinstance(rows[0], EsgBenchmark)


def test_benchmark_with_a_plain_year(client: Any, fixture_server: FixtureServer) -> None:
    """A four-digit ``year`` reaches the wire unchanged, as the direct-auth Rust test pins."""
    fixture_server.route("/esg-benchmark", load_fixture("esg_benchmark.json"))
    rows = client.esg.benchmark(year="2023")

    assert fixture_server.requests[0].target == "/esg-benchmark?year=2023"
    assert len(rows) == 1


def test_benchmark_year_is_keyword_only(client: Any, fixture_server: FixtureServer) -> None:
    """A positional year is a ``TypeError`` before any request, never a silent ``year``."""
    with pytest.raises(TypeError):
        client.esg.benchmark("2023")
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["disclosures", "ratings"])
def test_symbol_methods_require_the_symbol(client: Any, fixture_server: FixtureServer, method: str) -> None:
    """``disclosures`` and ``ratings`` without a symbol are a ``TypeError`` before any request."""
    with pytest.raises(TypeError):
        getattr(client.esg, method)()
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["disclosures", "ratings"])
@pytest.mark.parametrize(
    ("value", "message"),
    [
        ("", "symbol: value must not be empty or whitespace-only"),
        ("   ", "symbol: value must not be empty or whitespace-only"),
        ("AA\nPL", "symbol: value must not contain control characters"),
        ("AAPL,MSFT", "symbol: ticker must not contain a comma"),
    ],
)
def test_invalid_symbol_names_the_argument(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    value: str,
    message: str,
) -> None:
    """A blank, control-character, or comma-bearing ``symbol`` fails locally, prefixed with the argument name."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.esg, method)(value)
    error = raised.value
    assert str(error) == message
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("value", "message"),
    [
        ("", "year: value must not be empty or whitespace-only"),
        ("\t", "year: value must not be empty or whitespace-only"),
        ("20\n23", "year: value must not contain control characters"),
    ],
)
def test_invalid_benchmark_year_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, value: str, message: str
) -> None:
    """A blank or control-character ``year`` is rejected locally, prefixed with the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.esg.benchmark(year=value)
    error = raised.value
    assert str(error) == message
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_status_error_names_the_disclosures_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status carries the libfmp endpoint id, status, and body."""
    fixture_server.route("/esg-disclosures", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.esg.disclosures("AAPL")
    error = raised.value
    assert error.endpoint == "esg-disclosures"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_status_error_names_the_ratings_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A failing ratings route reports the ``esg-ratings`` endpoint id and the raw body."""
    fixture_server.route("/esg-ratings", b"not-json", status=500, content_type="text/plain")
    with pytest.raises(errors.FmpStatusError) as raised:
        client.esg.ratings("AAPL")
    assert raised.value.endpoint == "esg-ratings"
    assert raised.value.status == 500
    assert raised.value.body == "not-json"


def test_decode_error_names_the_benchmark_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-array body on the benchmark route surfaces as a decode error with the endpoint id."""
    fixture_server.route("/esg-benchmark", {})
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.esg.benchmark()
    assert raised.value.endpoint == "esg-benchmark"


def test_decode_error_names_the_disclosures_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A well-formed but wrongly shaped row on the disclosures route is a decode error."""
    fixture_server.route("/esg-disclosures", [{"symbol": "AAPL"}])
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.esg.disclosures("AAPL")
    assert raised.value.endpoint == "esg-disclosures"
