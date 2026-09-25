"""Runtime contract of ``client.directory``: symbol lists, taxonomies, negatives.

The expected targets are the ones the Rust ``directory_endpoints.rs`` and
``directory_taxonomy_endpoints.rs`` tests pin: eight query-less bare paths,
``cik-list`` with ``page`` then ``limit``, ``symbol-change`` with the
lowercase ``invalid`` flag then ``limit``, and ``available-exchanges`` with a
bare ``extended`` boolean. The domain's only temporal field is
``SymbolChange.date`` (a ``datetime.date``); it has no ``datetime.datetime``
column. ``no_of_transcripts`` is a provider numeric string and reaches Python
as ``str``. The negatives cover one invalid value per argument-kind family the
domain uses: ``page``, ``limit``, and the two ``bool``-typed kinds, which are
shape errors (``TypeError``) rather than validation errors.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.directory import (
    ActivelyTradingSymbol,
    AvailableCountry,
    AvailableExchange,
    AvailableIndustry,
    AvailableSector,
    CikListing,
    CompanySymbol,
    DirectoryNamespace,
    EarningsTranscriptAvailability,
    EtfSymbol,
    FinancialStatementSymbol,
    SymbolChange,
)


def test_directory_namespace_is_the_generated_type(client: Any) -> None:
    """``client.directory`` is a ``DirectoryNamespace`` exposing all eleven methods."""
    assert isinstance(client.directory, DirectoryNamespace)
    expected = {
        "company_symbols",
        "financial_statement_symbols",
        "cik_list",
        "symbol_changes",
        "etf_symbols",
        "actively_trading",
        "earnings_transcript_list",
        "available_exchanges",
        "available_sectors",
        "available_industries",
        "available_countries",
    }
    assert expected <= {name for name in dir(client.directory) if not name.startswith("_")}


def test_company_symbols_requests_the_bare_stock_list_path(client: Any, fixture_server: FixtureServer) -> None:
    """``company_symbols`` maps to ``/stock-list`` and decodes the ``companyName`` wire key."""
    fixture_server.route("/stock-list", load_fixture("directory_company_symbols.json"))
    rows = client.directory.company_symbols()

    assert fixture_server.requests[0].target == "/stock-list"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CompanySymbol)
    assert row.symbol == "URBANCO.BO"
    assert row.company_name == "Urban Company Limited"
    assert not hasattr(row, "name")


def test_financial_statement_symbols_decodes_both_currencies(client: Any, fixture_server: FixtureServer) -> None:
    """``financial_statement_symbols`` maps to ``/financial-statement-symbol-list``."""
    fixture_server.route("/financial-statement-symbol-list", load_fixture("directory_financial_statement_symbols.json"))
    rows = client.directory.financial_statement_symbols()

    assert fixture_server.requests[0].target == "/financial-statement-symbol-list"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, FinancialStatementSymbol)
    assert row.symbol == "RMES.CN"
    assert row.company_name == "Red Metal Resources Ltd."
    assert row.trading_currency == "CAD"
    assert row.reporting_currency == "USD"


def test_cik_list_with_page_and_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``cik_list`` encodes ``page`` then ``limit`` and keeps the zero-padded CIK as a string."""
    fixture_server.route("/cik-list", load_fixture("directory_cik_list.json"))
    rows = client.directory.cik_list(page=0, limit=10_001)

    assert fixture_server.requests[0].target == "/cik-list?page=0&limit=10001"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CikListing)
    assert row.cik == "0002137358"
    assert row.company_name == "Osotspa Public Co Limited/ADR"


def test_cik_list_with_limit_only(client: Any, fixture_server: FixtureServer) -> None:
    """``cik_list`` with ``limit`` alone sends only ``limit``."""
    fixture_server.route("/cik-list", load_fixture("directory_cik_list.json"))
    rows = client.directory.cik_list(limit=1_000)

    assert fixture_server.requests[0].target == "/cik-list?limit=1000"
    assert fixture_server.requests[0].query == {"limit": ["1000"]}
    assert len(rows) == 1


def test_cik_list_without_options_sends_the_bare_path(client: Any, fixture_server: FixtureServer) -> None:
    """``cik_list`` with nothing set requests the bare path and preserves an empty array."""
    fixture_server.route("/cik-list", load_fixture("directory_empty.json"))
    rows = client.directory.cik_list()

    assert fixture_server.requests[0].target == "/cik-list"
    assert fixture_server.requests[0].raw_query == ""
    assert rows == []


def test_symbol_changes_with_invalid_false_and_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``symbol_changes`` encodes ``invalid`` as lowercase ``false`` then ``limit``, and decodes the date."""
    fixture_server.route("/symbol-change", load_fixture("directory_symbol_changes.json"))
    rows = client.directory.symbol_changes(invalid=False, limit=100)

    assert fixture_server.requests[0].target == "/symbol-change?invalid=false&limit=100"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SymbolChange)
    assert row.date == datetime.date(2026, 7, 28)
    assert row.company_name == "Yarrow Bioscience, Inc. Common Stock"
    assert row.old_symbol == "VYNE"
    assert row.new_symbol == "YARW"


def test_symbol_changes_with_invalid_true_only(client: Any, fixture_server: FixtureServer) -> None:
    """``symbol_changes`` encodes ``invalid=True`` as the lowercase string ``true``."""
    fixture_server.route("/symbol-change", load_fixture("directory_symbol_changes.json"))
    rows = client.directory.symbol_changes(invalid=True)

    assert fixture_server.requests[0].target == "/symbol-change?invalid=true"
    assert fixture_server.requests[0].query == {"invalid": ["true"]}
    assert len(rows) == 1


def test_symbol_changes_without_options_sends_the_bare_path(client: Any, fixture_server: FixtureServer) -> None:
    """``symbol_changes`` with nothing set requests the bare path."""
    fixture_server.route("/symbol-change", load_fixture("directory_symbol_changes.json"))
    rows = client.directory.symbol_changes()

    assert fixture_server.requests[0].target == "/symbol-change"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1


def test_etf_symbols_decodes_the_name_wire_key(client: Any, fixture_server: FixtureServer) -> None:
    """``etf_symbols`` maps to ``/etf-list`` and keeps ``name`` rather than ``company_name``."""
    fixture_server.route("/etf-list", load_fixture("directory_etf_symbols.json"))
    rows = client.directory.etf_symbols()

    assert fixture_server.requests[0].target == "/etf-list"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EtfSymbol)
    assert row.symbol == "P60.SI"
    assert row.name == "MULTI-UNITS LUXEMBOURG - Lyxor MSCI AC Asia Pacific Ex Japan UCITS ETF"
    assert not hasattr(row, "company_name")


def test_actively_trading_decodes_the_name_wire_key(client: Any, fixture_server: FixtureServer) -> None:
    """``actively_trading`` maps to ``/actively-trading-list``."""
    fixture_server.route("/actively-trading-list", load_fixture("directory_actively_trading.json"))
    rows = client.directory.actively_trading()

    assert fixture_server.requests[0].target == "/actively-trading-list"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, ActivelyTradingSymbol)
    assert row.symbol == "URBANCO.BO"
    assert row.name == "Urban Company Limited"


def test_earnings_transcript_list_keeps_the_count_as_a_string(client: Any, fixture_server: FixtureServer) -> None:
    """``earnings_transcript_list`` maps to ``/earnings-transcript-list``; the count is a numeric string."""
    fixture_server.route("/earnings-transcript-list", load_fixture("directory_earnings_transcript_list.json"))
    rows = client.directory.earnings_transcript_list()

    assert fixture_server.requests[0].target == "/earnings-transcript-list"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EarningsTranscriptAvailability)
    assert row.symbol == "INBS"
    assert row.company_name == "Intelligent Bio Solutions Inc."
    assert row.no_of_transcripts == "6"
    assert isinstance(row.no_of_transcripts, str)


@pytest.mark.parametrize(
    ("extended", "target"),
    [
        pytest.param(None, "/available-exchanges", id="omitted"),
        pytest.param(False, "/available-exchanges?extended=false", id="false"),
        pytest.param(True, "/available-exchanges?extended=true", id="true"),
    ],
)
def test_available_exchanges_preserves_omitted_false_and_true(
    client: Any, fixture_server: FixtureServer, extended: bool | None, target: str
) -> None:
    """``available_exchanges`` sends ``extended`` only when set, as a lowercase boolean."""
    fixture_server.route("/available-exchanges", load_fixture("directory_available_exchanges.json"))
    kwargs: dict[str, Any] = {} if extended is None else {"extended": extended}
    rows = client.directory.available_exchanges(**kwargs)

    assert fixture_server.requests[0].target == target
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, AvailableExchange)
    assert row.exchange == "AMEX"
    assert row.name == "New York Stock Exchange Arca"
    assert row.country_name == "United States of America"
    assert row.country_code == "US"
    assert row.symbol_suffix == "N/A"
    assert row.delay == "Real-time"


def test_available_sectors_decodes_the_open_sector_value(client: Any, fixture_server: FixtureServer) -> None:
    """``available_sectors`` maps to ``/available-sectors``."""
    fixture_server.route("/available-sectors", load_fixture("directory_available_sectors.json"))
    rows = client.directory.available_sectors()

    assert fixture_server.requests[0].target == "/available-sectors"
    assert len(rows) == 1
    assert isinstance(rows[0], AvailableSector)
    assert rows[0].sector == "Basic Materials"


def test_available_industries_decodes_the_open_industry_value(client: Any, fixture_server: FixtureServer) -> None:
    """``available_industries`` maps to ``/available-industries``."""
    fixture_server.route("/available-industries", load_fixture("directory_available_industries.json"))
    rows = client.directory.available_industries()

    assert fixture_server.requests[0].target == "/available-industries"
    assert len(rows) == 1
    assert isinstance(rows[0], AvailableIndustry)
    assert rows[0].industry == "Steel"


def test_available_countries_decodes_the_country_code(client: Any, fixture_server: FixtureServer) -> None:
    """``available_countries`` maps to ``/available-countries``."""
    fixture_server.route("/available-countries", load_fixture("directory_available_countries.json"))
    rows = client.directory.available_countries()

    assert fixture_server.requests[0].target == "/available-countries"
    assert len(rows) == 1
    assert isinstance(rows[0], AvailableCountry)
    assert rows[0].country == "FK"


@pytest.mark.parametrize(
    "method",
    [
        "company_symbols",
        "financial_statement_symbols",
        "etf_symbols",
        "actively_trading",
        "earnings_transcript_list",
        "available_sectors",
        "available_industries",
        "available_countries",
    ],
)
def test_query_less_methods_preserve_empty_arrays(client: Any, fixture_server: FixtureServer, method: str) -> None:
    """Each query-less method returns ``[]`` for the server's default empty array and takes no arguments."""
    assert getattr(client.directory, method)() == []
    assert len(fixture_server.requests) == 1
    assert fixture_server.requests[0].raw_query == ""
    with pytest.raises(TypeError):
        getattr(client.directory, method)("AAPL")


@pytest.mark.parametrize(
    ("method", "keyword", "value", "message"),
    [
        pytest.param("cik_list", "page", -1, "page: must be an integer from 0 through 4294967295", id="page"),
        pytest.param("cik_list", "limit", -1, "limit: ", id="limit-cik"),
        pytest.param("symbol_changes", "limit", 4_294_967_296, "limit: ", id="limit-symbol-changes"),
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
    """Each numeric kind family fails locally with ``FmpValidationError`` prefixed by the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.directory, method)(**{keyword: value})
    error = raised.value
    assert str(error).startswith(message)
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize("value", ["false", 0, [True]], ids=["string", "int", "list"])
def test_bool_typed_keywords_reject_non_bool_shapes(client: Any, fixture_server: FixtureServer, value: Any) -> None:
    """``invalid`` (``true_false_flag``) and ``extended`` (``boolean``) accept only ``bool``."""
    with pytest.raises(TypeError):
        client.directory.symbol_changes(invalid=value)
    with pytest.raises(TypeError):
        client.directory.available_exchanges(extended=value)
    assert fixture_server.requests == []


def test_query_methods_are_keyword_only(client: Any, fixture_server: FixtureServer) -> None:
    """The three query methods have only optional keywords; a positional value is a ``TypeError``."""
    with pytest.raises(TypeError):
        client.directory.cik_list(0)
    with pytest.raises(TypeError):
        client.directory.symbol_changes(False)
    with pytest.raises(TypeError):
        client.directory.available_exchanges(True)
    assert fixture_server.requests == []


def test_status_error_carries_the_directory_endpoint_id(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on ``cik_list`` maps to ``FmpStatusError`` naming ``cik-list``."""
    fixture_server.route("/cik-list", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.directory.cik_list(page=0)
    error = raised.value
    assert error.endpoint == "cik-list"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_decode_error_on_a_non_json_body(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-JSON 200 body maps to ``FmpDecodeError`` and names the endpoint."""
    fixture_server.route("/available-sectors", b"not-json", content_type="text/plain")
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.directory.available_sectors()
    assert raised.value.endpoint == "available-sectors"
