"""Runtime contract of ``client.insider_trading``: reference lookups, filings, negatives.

The expected targets are the ones the Rust
``insider_trading_reference_endpoints.rs`` and
``insider_trading_beneficial_ownership_endpoint.rs`` tests pin. The negatives
cover one invalid value per argument-kind family the domain uses: ``ticker``,
``cik``, ``search_term``, ``transaction_type_code``, ``date``, ``page``, and
``limit``. The failure routes prove the status and decode error mapping names
the ``insider-trading/...`` endpoint id.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.insider_trading import (
    BeneficialOwnershipAcquisition,
    InsiderReportingName,
    InsiderTradeStatistics,
    InsiderTransactionType,
)

APPLE_CIK = "0000320193"
SPACED_SYMBOL = "BRK.B / Class A"
SPACED_SYMBOL_ENCODED = "BRK.B+%2F+Class+A"
SPACED_NAME = "Zuckerberg, Mark / Meta"
SPACED_NAME_ENCODED = "Zuckerberg%2C+Mark+%2F+Meta"
U32_MAX = 4_294_967_295


def test_search_reporting_names_form_encodes_the_search_term(client: Any, fixture_server: FixtureServer) -> None:
    """``search_reporting_names`` accepts commas and slashes in ``name`` and form-encodes them."""
    fixture_server.route("/insider-trading/reporting-name", load_fixture("insider_reporting_names.json"))
    rows = client.insider_trading.search_reporting_names(SPACED_NAME)

    assert fixture_server.requests[0].target == f"/insider-trading/reporting-name?name={SPACED_NAME_ENCODED}"
    assert fixture_server.requests[0].query["name"] == [SPACED_NAME]
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, InsiderReportingName)
    assert row.reporting_cik == "0001548760"
    assert row.reporting_name == "Zuckerberg Mark"


def test_search_reporting_names_accepts_the_keyword_form(client: Any, fixture_server: FixtureServer) -> None:
    """``search_reporting_names`` takes ``name`` by keyword as well as by position."""
    fixture_server.route("/insider-trading/reporting-name", load_fixture("insider_reporting_names.json"))
    rows = client.insider_trading.search_reporting_names(name="Zuckerberg")

    assert fixture_server.requests[0].target == "/insider-trading/reporting-name?name=Zuckerberg"
    assert len(rows) == 1


def test_transaction_types_sends_the_bare_hyphenated_path(client: Any, fixture_server: FixtureServer) -> None:
    """``transaction_types`` hits the single-segment path with no query and takes no arguments."""
    fixture_server.route("/insider-trading-transaction-type", load_fixture("insider_transaction_types.json"))
    rows = client.insider_trading.transaction_types()

    assert fixture_server.requests[0].target == "/insider-trading-transaction-type"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, InsiderTransactionType)
    assert row.transaction_type == "A-Award"
    with pytest.raises(TypeError):
        client.insider_trading.transaction_types("A-Award")


def test_trade_statistics_form_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``trade_statistics`` sends the ticker as the only parameter and decodes the quarterly row."""
    fixture_server.route("/insider-trading/statistics", load_fixture("insider_trade_statistics.json"))
    rows = client.insider_trading.trade_statistics(SPACED_SYMBOL)

    assert fixture_server.requests[0].target == f"/insider-trading/statistics?symbol={SPACED_SYMBOL_ENCODED}"
    assert fixture_server.requests[0].query["symbol"] == [SPACED_SYMBOL]
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, InsiderTradeStatistics)
    assert row.symbol == "AAPL"
    assert row.cik == APPLE_CIK
    assert row.year == 2026
    assert row.quarter == 2
    assert row.acquired_transactions == 7
    assert row.disposed_transactions == 40
    assert row.acquired_disposed_ratio == pytest.approx(0.175)
    assert row.total_acquired == 303_199
    assert row.total_disposed == 927_380
    assert row.average_acquired == pytest.approx(43_314.1429)
    assert row.average_disposed == pytest.approx(23_184.5)
    assert row.total_purchases == 0
    assert row.total_sales == 14


def test_trade_statistics_with_a_plain_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``trade_statistics`` sends a plain ticker verbatim, by keyword too."""
    fixture_server.route("/insider-trading/statistics", load_fixture("insider_trade_statistics.json"))
    rows = client.insider_trading.trade_statistics(symbol="AAPL")

    assert fixture_server.requests[0].target == "/insider-trading/statistics?symbol=AAPL"
    assert len(rows) == 1


def test_beneficial_ownership_acquisitions_with_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``beneficial_ownership_acquisitions`` encodes ``symbol`` then ``limit`` and decodes the filing."""
    fixture_server.route(
        "/acquisition-of-beneficial-ownership", load_fixture("beneficial_ownership_acquisitions.json")
    )
    rows = client.insider_trading.beneficial_ownership_acquisitions("AAPL", limit=0)

    assert fixture_server.requests[0].target == "/acquisition-of-beneficial-ownership?symbol=AAPL&limit=0"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, BeneficialOwnershipAcquisition)
    assert row.cik == APPLE_CIK
    assert row.symbol == "AAPL"
    assert row.filing_date == datetime.date(2026, 4, 29)
    assert row.accepted_date == datetime.date(2026, 4, 29)
    assert row.cusip == "037833100"
    assert row.name_of_reporting_person == "Vanguard Capital Management"
    assert row.citizenship_or_place_of_organization == "PENNSYLVANIA"
    assert row.sole_voting_power == "0"
    assert row.shared_voting_power == "0"
    assert row.sole_dispositive_power == "0"
    assert row.shared_dispositive_power == "0"
    assert row.amount_beneficially_owned == "1099168953"
    assert row.percent_of_class == "7.48"
    assert row.type_of_reporting_person == "IA"
    assert row.url.startswith("https://www.sec.gov/Archives/edgar/data/320193/")


def test_beneficial_ownership_acquisitions_decode_null_members_as_none(
    client: Any, fixture_server: FixtureServer
) -> None:
    """Issue #368: a null ``cusip``, citizenship, or shared voting power decodes as ``None``."""
    body = load_fixture("beneficial_ownership_acquisitions.json")
    body[0].update(cusip=None, citizenshipOrPlaceOfOrganization=None, sharedVotingPower=None)
    fixture_server.route("/acquisition-of-beneficial-ownership", body)
    rows = client.insider_trading.beneficial_ownership_acquisitions("AAPL")

    assert rows[0].cusip is None
    assert rows[0].citizenship_or_place_of_organization is None
    assert rows[0].shared_voting_power is None
    assert rows[0].sole_voting_power == "0"


def test_beneficial_ownership_acquisitions_with_a_spaced_symbol_and_the_u32_limit(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``beneficial_ownership_acquisitions`` form-encodes the ticker and passes ``2**32 - 1`` unchanged."""
    fixture_server.route(
        "/acquisition-of-beneficial-ownership", load_fixture("beneficial_ownership_acquisitions.json")
    )
    rows = client.insider_trading.beneficial_ownership_acquisitions(SPACED_SYMBOL, limit=U32_MAX)

    assert (
        fixture_server.requests[0].target
        == f"/acquisition-of-beneficial-ownership?symbol={SPACED_SYMBOL_ENCODED}&limit={U32_MAX}"
    )
    assert fixture_server.requests[0].query["symbol"] == [SPACED_SYMBOL]
    assert len(rows) == 1


def test_beneficial_ownership_acquisitions_without_limit_sends_only_the_symbol(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``beneficial_ownership_acquisitions`` with no ``limit`` sends only the ticker and rejects a positional limit."""
    fixture_server.route(
        "/acquisition-of-beneficial-ownership", load_fixture("beneficial_ownership_acquisitions.json")
    )
    rows = client.insider_trading.beneficial_ownership_acquisitions("AAPL")

    assert fixture_server.requests[0].target == "/acquisition-of-beneficial-ownership?symbol=AAPL"
    assert len(rows) == 1
    with pytest.raises(TypeError):
        client.insider_trading.beneficial_ownership_acquisitions("AAPL", 0)


@pytest.mark.parametrize(
    ("method", "arguments", "keyword", "message"),
    [
        pytest.param(
            "trade_statistics",
            ("AAPL,MSFT",),
            "symbol",
            "symbol: ticker must not contain a comma",
            id="ticker-comma",
        ),
        pytest.param(
            "beneficial_ownership_acquisitions",
            ("   ",),
            "symbol",
            "symbol: value must not be empty or whitespace-only",
            id="ticker-whitespace",
        ),
        pytest.param(
            "search_reporting_names",
            ("",),
            "name",
            "name: value must not be empty or whitespace-only",
            id="search-term-empty",
        ),
        pytest.param(
            "search_reporting_names",
            ("Zuckerberg\tMark",),
            "name",
            "name: value must not contain control characters",
            id="search-term-control",
        ),
    ],
)
def test_invalid_positional_values_name_the_argument(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    arguments: tuple[Any, ...],
    keyword: str,
    message: str,
) -> None:
    """Each required-argument kind fails locally with ``FmpValidationError`` prefixed by the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.insider_trading, method)(*arguments)
    error = raised.value
    assert str(error).startswith(message)
    assert str(error).startswith(f"{keyword}: ")
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("method", "arguments", "keyword", "value", "message"),
    [
        pytest.param(
            "search_trades",
            (),
            "symbol",
            "AAPL,MSFT",
            "symbol: ticker must not contain a comma",
            id="ticker",
        ),
        pytest.param(
            "search_trades",
            (),
            "reporting_cik",
            "",
            "reporting_cik: value must not be empty or whitespace-only",
            id="reporting-cik",
        ),
        pytest.param(
            "search_trades",
            (),
            "company_cik",
            " \t ",
            "company_cik: value must not be empty or whitespace-only",
            id="company-cik",
        ),
        pytest.param(
            "search_trades",
            (),
            "transaction_type",
            "",
            "transaction_type: value must not be empty or whitespace-only",
            id="transaction-type",
        ),
        pytest.param(
            "latest_trades",
            (),
            "date",
            "27/01/2026",
            "date: value must be a valid YYYY-MM-DD date",
            id="date",
        ),
        pytest.param(
            "latest_trades",
            (),
            "page",
            -1,
            "page: must be an integer from 0 through 4294967295",
            id="page",
        ),
        pytest.param(
            "search_trades",
            (),
            "limit",
            U32_MAX + 1,
            "limit: must be an integer from 0 through 4294967295",
            id="limit",
        ),
        pytest.param(
            "beneficial_ownership_acquisitions",
            ("AAPL",),
            "limit",
            -1,
            "limit: must be an integer from 0 through 4294967295",
            id="limit-on-filings",
        ),
    ],
)
def test_invalid_keyword_values_name_the_argument(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    arguments: tuple[Any, ...],
    keyword: str,
    value: Any,
    message: str,
) -> None:
    """Each optional-setter kind fails locally with ``FmpValidationError`` prefixed by the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.insider_trading, method)(*arguments, **{keyword: value})
    error = raised.value
    assert str(error).startswith(message)
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_wrong_argument_types_raise_type_error(client: Any, fixture_server: FixtureServer) -> None:
    """An ``int`` ticker, a ``str`` page, or an ``int`` date is a shape error, reported by pyo3 as ``TypeError``."""
    with pytest.raises(TypeError):
        client.insider_trading.trade_statistics(320193)
    with pytest.raises(TypeError):
        client.insider_trading.latest_trades(page="0")
    with pytest.raises(TypeError):
        client.insider_trading.latest_trades(date=20260127)
    assert fixture_server.requests == []


def test_status_error_carries_the_nested_endpoint_id(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on ``search_trades`` names the two-segment endpoint id."""
    fixture_server.route("/insider-trading/search", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.insider_trading.search_trades(symbol="AAPL")
    error = raised.value
    assert error.endpoint == "insider-trading/search"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_decode_error_on_a_non_json_body(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-JSON 200 body maps to ``FmpDecodeError`` and names the hyphenated endpoint."""
    fixture_server.route("/insider-trading-transaction-type", b"not-json", content_type="text/plain")
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.insider_trading.transaction_types()
    assert raised.value.endpoint == "insider-trading-transaction-type"
