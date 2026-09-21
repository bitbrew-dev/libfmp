"""Runtime contract of ``client.transcripts`` for the three earnings-transcript methods.

One test per argument shape routes the documented fixture body, calls the
method, and asserts the exact request target plus a few typed fields
(including the ``datetime.date`` ones). The expected targets are the ones the
Rust ``transcript_content_endpoints.rs`` and
``transcript_discovery_endpoints.rs`` tests pin. The negatives cover every
argument kind the domain has (``ticker``, ``year``, ``quarter``, ``limit``,
and ``page``) plus the structured status and decode failures.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.transcripts import (
    EarningsTranscript,
    EarningsTranscriptDate,
    LatestEarningsTranscript,
    TranscriptsNamespace,
)

U32_MESSAGE = "must be an integer from 0 through 4294967295"


def test_transcripts_namespace_is_the_generated_type(client: Any) -> None:
    """``client.transcripts`` is the generated flat namespace class."""
    assert isinstance(client.transcripts, TranscriptsNamespace)


def test_latest_earnings_transcripts_with_no_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_earnings_transcripts`` maps to the bare latest path and decodes the metadata row."""
    fixture_server.route("/earning-call-transcript-latest", load_fixture("latest_earnings_transcripts.json"))
    rows = client.transcripts.latest_earnings_transcripts()

    assert fixture_server.requests[0].target == "/earning-call-transcript-latest"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, LatestEarningsTranscript)
    assert row.symbol == "VLO"
    assert row.period == "Q2"
    assert row.fiscal_year == 2026
    assert row.date == datetime.date(2026, 7, 30)


def test_latest_earnings_transcripts_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_earnings_transcripts`` encodes ``limit`` then ``page``."""
    fixture_server.route("/earning-call-transcript-latest", load_fixture("latest_earnings_transcripts.json"))
    rows = client.transcripts.latest_earnings_transcripts(limit=100, page=0)

    assert fixture_server.requests[0].target == "/earning-call-transcript-latest?limit=100&page=0"
    assert len(rows) == 1
    assert isinstance(rows[0].date, datetime.date)


@pytest.mark.parametrize(
    ("keywords", "target"),
    [
        ({"limit": 100}, "/earning-call-transcript-latest?limit=100"),
        ({"page": 0}, "/earning-call-transcript-latest?page=0"),
        ({"limit": 101, "page": 100}, "/earning-call-transcript-latest?limit=101&page=100"),
    ],
)
def test_latest_earnings_transcripts_options_are_independent(
    client: Any, fixture_server: FixtureServer, keywords: dict[str, int], target: str
) -> None:
    """Each latest option is sent alone when the other one is left out."""
    fixture_server.route("/earning-call-transcript-latest", load_fixture("latest_earnings_transcripts.json"))
    rows = client.transcripts.latest_earnings_transcripts(**keywords)

    assert fixture_server.requests[0].target == target
    assert len(rows) == 1


def test_earnings_transcript_with_required_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``earnings_transcript`` encodes ``symbol``, ``year``, ``quarter`` and decodes the full transcript row."""
    fixture_server.route("/earning-call-transcript", load_fixture("earnings_transcript.json"))
    rows = client.transcripts.earnings_transcript("AAPL", 2020, 3)

    assert fixture_server.requests[0].target == "/earning-call-transcript?symbol=AAPL&year=2020&quarter=3"
    assert fixture_server.requests[0].query == {"symbol": ["AAPL"], "year": ["2020"], "quarter": ["3"]}
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EarningsTranscript)
    assert row.symbol == "AAPL"
    assert row.period == "Q3"
    assert row.year == 2020
    assert row.date == datetime.date(2020, 7, 30)
    assert "\nTejas Gala:" in row.content
    assert row.content.endswith("Aft...")


def test_earnings_transcript_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``earnings_transcript`` appends the optional ``limit`` after the required trio."""
    fixture_server.route("/earning-call-transcript", load_fixture("earnings_transcript.json"))
    rows = client.transcripts.earnings_transcript("AAPL", 2020, 3, limit=1)

    assert fixture_server.requests[0].target == "/earning-call-transcript?symbol=AAPL&year=2020&quarter=3&limit=1"
    assert len(rows) == 1
    assert isinstance(rows[0].date, datetime.date)


def test_earnings_transcript_accepts_the_u32_limit_boundary(client: Any, fixture_server: FixtureServer) -> None:
    """The largest unsigned 32-bit ``limit`` reaches the wire unchanged."""
    fixture_server.route("/earning-call-transcript", load_fixture("earnings_transcript.json"))
    client.transcripts.earnings_transcript("AAPL", 2020, 3, limit=4_294_967_295)

    assert (
        fixture_server.requests[0].target
        == "/earning-call-transcript?symbol=AAPL&year=2020&quarter=3&limit=4294967295"
    )


def test_earnings_transcript_accepts_keywords_for_the_required_trio(
    client: Any, fixture_server: FixtureServer
) -> None:
    """The required ``symbol``, ``year``, ``quarter`` may be passed as keywords in any order."""
    fixture_server.route("/earning-call-transcript", load_fixture("earnings_transcript.json"))
    rows = client.transcripts.earnings_transcript(quarter=3, year=2020, symbol="AAPL")

    assert fixture_server.requests[0].target == "/earning-call-transcript?symbol=AAPL&year=2020&quarter=3"
    assert len(rows) == 1


def test_earnings_transcript_dates_with_a_ticker(client: Any, fixture_server: FixtureServer) -> None:
    """``earnings_transcript_dates`` sends only ``symbol`` and decodes the numeric quarter row."""
    fixture_server.route("/earning-call-transcript-dates", load_fixture("earnings_transcript_dates.json"))
    rows = client.transcripts.earnings_transcript_dates("AAPL")

    assert fixture_server.requests[0].target == "/earning-call-transcript-dates?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EarningsTranscriptDate)
    assert row.quarter == 2
    assert row.fiscal_year == 2026
    assert row.date == datetime.date(2026, 4, 30)


def test_optionals_are_keyword_only(client: Any, fixture_server: FixtureServer) -> None:
    """A positional limit or page is a ``TypeError``, never a silent option."""
    with pytest.raises(TypeError):
        client.transcripts.latest_earnings_transcripts(100)
    with pytest.raises(TypeError):
        client.transcripts.earnings_transcript("AAPL", 2020, 3, 1)
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("method", "args"),
    [
        pytest.param("earnings_transcript", ("AAPL", 2020), id="earnings_transcript"),
        pytest.param("earnings_transcript_dates", (), id="earnings_transcript_dates"),
    ],
)
def test_missing_required_arguments_are_type_errors(
    client: Any, fixture_server: FixtureServer, method: str, args: tuple[Any, ...]
) -> None:
    """Leaving out a required argument is a ``TypeError`` before any request."""
    with pytest.raises(TypeError):
        getattr(client.transcripts, method)(*args)
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["earnings_transcript", "earnings_transcript_dates"])
@pytest.mark.parametrize("value", ["", "   ", "AA\nPL"])
def test_invalid_symbol_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, value: str
) -> None:
    """A blank or control-character ``symbol`` is rejected locally, prefixed with the argument name."""
    args = (value, 2020, 3) if method == "earnings_transcript" else (value,)
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.transcripts, method)(*args)
    error = raised.value
    assert str(error).startswith("symbol: value must not ")
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize("year", [-1, 4_294_967_296])
def test_out_of_range_year_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, year: int
) -> None:
    """A year outside the provider's unsigned 32-bit range is rejected locally."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.transcripts.earnings_transcript("AAPL", year, 3)
    assert str(raised.value) == f"year: {U32_MESSAGE}"
    assert fixture_server.requests == []


@pytest.mark.parametrize("quarter", [0, 5, -1])
def test_out_of_range_quarter_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, quarter: int
) -> None:
    """A quarter outside 1 through 4 is rejected locally; a float quarter is a ``TypeError``."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.transcripts.earnings_transcript("AAPL", 2020, quarter)
    assert str(raised.value) == "quarter: quarter must be an integer from 1 through 4"
    assert fixture_server.requests == []
    with pytest.raises(TypeError):
        client.transcripts.earnings_transcript("AAPL", 2020, 3.0)


@pytest.mark.parametrize("keyword", ["limit", "page"])
@pytest.mark.parametrize("value", [-1, 4_294_967_296])
def test_out_of_range_latest_options_name_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, keyword: str, value: int
) -> None:
    """A ``limit`` or ``page`` outside the unsigned 32-bit range is rejected locally."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.transcripts.latest_earnings_transcripts(**{keyword: value})
    error = raised.value
    assert str(error) == f"{keyword}: {U32_MESSAGE}"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_out_of_range_transcript_limit_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """``earnings_transcript`` validates its ``limit`` the same way once the trio has been accepted."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.transcripts.earnings_transcript("AAPL", 2020, 3, limit=-1)
    assert str(raised.value) == f"limit: {U32_MESSAGE}"
    assert fixture_server.requests == []


def test_wrong_shapes_are_type_errors(client: Any, fixture_server: FixtureServer) -> None:
    """An ``int`` symbol, a ``str`` year, or a ``float`` limit is a shape error reported as ``TypeError``."""
    with pytest.raises(TypeError):
        client.transcripts.earnings_transcript_dates(123)
    with pytest.raises(TypeError):
        client.transcripts.earnings_transcript("AAPL", "2020", 3)
    with pytest.raises(TypeError):
        client.transcripts.latest_earnings_transcripts(limit=1.5)
    assert fixture_server.requests == []


def test_status_error_names_the_latest_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status carries the libfmp endpoint id, status, and body."""
    fixture_server.route("/earning-call-transcript-latest", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.transcripts.latest_earnings_transcripts()
    error = raised.value
    assert error.endpoint == "earning-call-transcript-latest"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_status_error_names_the_transcript_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A failing transcript route reports the ``earning-call-transcript`` endpoint id and the raw body."""
    fixture_server.route("/earning-call-transcript", b"not-json", status=500, content_type="text/plain")
    with pytest.raises(errors.FmpStatusError) as raised:
        client.transcripts.earnings_transcript("AAPL", 2020, 3)
    assert raised.value.endpoint == "earning-call-transcript"
    assert raised.value.status == 500
    assert raised.value.body == "not-json"


def test_decode_error_names_the_dates_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-array body on the dates route surfaces as a decode error with the endpoint id."""
    fixture_server.route("/earning-call-transcript-dates", {})
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.transcripts.earnings_transcript_dates("AAPL")
    assert raised.value.endpoint == "earning-call-transcript-dates"


def test_decode_error_names_the_transcript_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A well-formed but wrongly shaped row on the transcript route is a decode error."""
    fixture_server.route("/earning-call-transcript", [{"symbol": "AAPL"}])
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.transcripts.earnings_transcript("AAPL", 2020, 3)
    assert raised.value.endpoint == "earning-call-transcript"
