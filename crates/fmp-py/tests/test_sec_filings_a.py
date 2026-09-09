"""Runtime contract of ``client.sec_filings`` for the five filing feeds.

Covers ``latest_8k``, ``latest``, ``by_form_type``, ``by_symbol``, and
``by_cik``: one test per method routes the documented fixture body, calls the
method with one argument shape, and asserts the exact request target plus a
few typed fields (including the ``datetime.datetime`` ones). The expected
targets are the ones the Rust ``sec_filing_feeds_endpoints.rs`` test pins.
The company lookups, the classifications, and the negative cases live in
``test_sec_filings_b.py``.
"""

import datetime
from typing import Any

from conftest import FixtureServer, load_fixture
from fmp.sec_filings import SecFiling, SecFilingsNamespace

FROM_2024_01_01 = datetime.date(2024, 1, 1)
TO_2024_03_01 = datetime.date(2024, 3, 1)
U32_MAX = 4_294_967_295


def test_sec_filings_namespace_is_the_generated_type(client: Any) -> None:
    """``client.sec_filings`` is the generated flat namespace class."""
    assert isinstance(client.sec_filings, SecFilingsNamespace)


def test_latest_8k_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_8k`` encodes ``from``, ``to``, ``page``, ``limit`` in that order."""
    fixture_server.route("/sec-filings-8k", load_fixture("latest_8k_sec_filings.json"))
    rows = client.sec_filings.latest_8k(FROM_2024_01_01, TO_2024_03_01, page=0, limit=U32_MAX)

    assert fixture_server.requests[0].target == "/sec-filings-8k?from=2024-01-01&to=2024-03-01&page=0&limit=4294967295"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SecFiling)
    assert row.symbol == "SUNE"
    assert row.cik == "0000022701"
    assert row.filing_date == datetime.datetime(2024, 3, 4, 0, 0, 0)
    assert row.accepted_date == datetime.datetime(2024, 3, 1, 22, 47, 48)
    assert row.form_type == "8-K"
    assert row.has_financials is None
    assert row.link.endswith("-index.htm")
    assert row.final_link.endswith("_8k.htm")


def test_latest_8k_accepts_iso_strings_and_omits_pagination(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_8k`` takes ``from_`` and ``to`` as ISO strings and sends only the dates."""
    fixture_server.route("/sec-filings-8k", load_fixture("latest_8k_sec_filings.json"))
    rows = client.sec_filings.latest_8k("2024-01-01", "2024-03-01")

    assert fixture_server.requests[0].target == "/sec-filings-8k?from=2024-01-01&to=2024-03-01"
    assert fixture_server.requests[0].query["from"] == ["2024-01-01"]
    assert len(rows) == 1
    assert isinstance(rows[0], SecFiling)


def test_latest_without_options(client: Any, fixture_server: FixtureServer) -> None:
    """``latest`` maps to ``/sec-filings-financials`` and decodes ``has_financials`` as a bool."""
    fixture_server.route("/sec-filings-financials", load_fixture("latest_sec_filings.json"))
    rows = client.sec_filings.latest(FROM_2024_01_01, TO_2024_03_01)

    assert fixture_server.requests[0].target == "/sec-filings-financials?from=2024-01-01&to=2024-03-01"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SecFiling)
    assert row.symbol == "DNN"
    assert row.cik == "0001063259"
    assert row.form_type == "6-K"
    assert row.has_financials is True
    assert row.filing_date == datetime.datetime(2024, 3, 1, 0, 0, 0)
    assert row.accepted_date == datetime.datetime(2024, 3, 1, 16, 52, 35)


def test_latest_with_page_only(client: Any, fixture_server: FixtureServer) -> None:
    """``latest`` sends ``page`` without ``limit`` when only the page is given."""
    fixture_server.route("/sec-filings-financials", load_fixture("latest_sec_filings.json"))
    rows = client.sec_filings.latest(from_=FROM_2024_01_01, to=TO_2024_03_01, page=3)

    assert fixture_server.requests[0].target == "/sec-filings-financials?from=2024-01-01&to=2024-03-01&page=3"
    assert len(rows) == 1


def test_by_form_type_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``by_form_type`` encodes ``formType`` first and accepts a zero ``limit``."""
    fixture_server.route("/sec-filings-search/form-type", load_fixture("sec_filings_by_form_type.json"))
    rows = client.sec_filings.by_form_type("8-K", FROM_2024_01_01, TO_2024_03_01, page=0, limit=0)

    assert (
        fixture_server.requests[0].target
        == "/sec-filings-search/form-type?formType=8-K&from=2024-01-01&to=2024-03-01&page=0&limit=0"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SecFiling)
    assert row.symbol == "SUNE"
    assert row.form_type == "8-K"
    assert row.has_financials is None
    assert row.accepted_date == datetime.datetime(2024, 3, 1, 22, 47, 48)


def test_by_symbol_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``by_symbol`` form-encodes the ticker and sends only the required parameters."""
    fixture_server.route("/sec-filings-search/symbol", load_fixture("sec_filings_by_symbol.json"))
    rows = client.sec_filings.by_symbol("BRK.B / Class A", FROM_2024_01_01, TO_2024_03_01)

    assert (
        fixture_server.requests[0].target
        == "/sec-filings-search/symbol?symbol=BRK.B+%2F+Class+A&from=2024-01-01&to=2024-03-01"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SecFiling)
    assert row.symbol == "AAPL"
    assert row.cik == "0000320193"
    assert row.form_type == "4"
    assert row.has_financials is None
    assert row.filing_date == datetime.datetime(2024, 3, 1, 0, 0, 0)
    assert row.final_link.endswith("wk-form4_1709336196.xml")


def test_by_symbol_with_limit_only(client: Any, fixture_server: FixtureServer) -> None:
    """``by_symbol`` sends ``limit`` without ``page`` when only the limit is given."""
    fixture_server.route("/sec-filings-search/symbol", load_fixture("sec_filings_by_symbol.json"))
    rows = client.sec_filings.by_symbol("AAPL", "2024-01-01", "2024-03-01", limit=25)

    assert fixture_server.requests[0].target == "/sec-filings-search/symbol?symbol=AAPL&from=2024-01-01&to=2024-03-01&limit=25"
    assert len(rows) == 1


def test_by_cik_with_maximal_pagination(client: Any, fixture_server: FixtureServer) -> None:
    """``by_cik`` keeps the leading zeroes of the CIK and accepts ``u32::MAX`` page and limit."""
    fixture_server.route("/sec-filings-search/cik", load_fixture("sec_filings_by_cik.json"))
    rows = client.sec_filings.by_cik("0000320193", FROM_2024_01_01, TO_2024_03_01, page=U32_MAX, limit=U32_MAX)

    assert (
        fixture_server.requests[0].target
        == "/sec-filings-search/cik?cik=0000320193&from=2024-01-01&to=2024-03-01&page=4294967295&limit=4294967295"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SecFiling)
    assert row.cik == "0000320193"
    assert row.symbol == "AAPL"
    assert row.accepted_date == datetime.datetime(2024, 3, 1, 18, 36, 45)
    assert row.link.endswith("0000320193-24-000039-index.htm")


def test_filing_feeds_decode_an_empty_array(client: Any, fixture_server: FixtureServer) -> None:
    """A documented empty body decodes to an empty list on every feed."""
    fixture_server.route("/sec-filings-8k", [])
    fixture_server.route("/sec-filings-search/cik", [])

    assert client.sec_filings.latest_8k(FROM_2024_01_01, TO_2024_03_01) == []
    assert client.sec_filings.by_cik("0000320193", FROM_2024_01_01, TO_2024_03_01) == []
    assert [request.path for request in fixture_server.requests] == ["/sec-filings-8k", "/sec-filings-search/cik"]
