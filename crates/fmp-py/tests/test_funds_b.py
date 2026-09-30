"""Runtime contract of ``client.funds`` for the disclosure queries and the negatives.

Covers ``disclosures`` (required ticker, year, quarter plus the
keyword-only ``cik``), ``search_disclosure_holders`` (a free-text
name), and ``disclosure_dates`` (ticker plus keyword-only ``cik``),
whose rows are the ``Form13fFilingDate`` model shared with
``fmp.institutional_ownership`` because libfmp aliases the two. The expected
targets are the ones the Rust ``fund_disclosure_endpoints.rs`` test pins.
The negatives prove each argument kind fails locally with
``FmpValidationError`` prefixed by the argument name, and that status and
decode failures stay structured.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.funds import FundDisclosure, FundDisclosureSearchResult
from fmp.institutional_ownership import Form13fFilingDate

FEDERATED = "Federated Hermes Government Income Securities, Inc."


def test_fund_disclosures_with_cik(client: Any, fixture_server: FixtureServer) -> None:
    """``disclosures`` encodes ``symbol``, ``year``, ``quarter``, then ``cik`` and decodes the position row."""
    fixture_server.route("/funds/disclosure", load_fixture("fund_disclosures.json"))
    rows = client.funds.disclosures("VWO", 2023, 4, cik="0000857489")

    assert fixture_server.requests[0].target == "/funds/disclosure?symbol=VWO&year=2023&quarter=4&cik=0000857489"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, FundDisclosure)
    assert row.cik == "0000857489"
    assert row.date == datetime.date(2023, 10, 31)
    assert row.accepted_date == datetime.datetime(2023, 12, 28, 9, 26, 13)
    assert row.symbol == "000089.SZ"
    assert row.name == "Shenzhen Airport Co Ltd"
    assert row.lei == "3003009W045RIKRBZI44"
    assert row.cusip == "N/A"
    assert row.isin == "CNE000000VK1"
    assert row.balance == 2_438_784
    assert row.units == "NS"
    assert row.currency_code == "CNY"
    assert row.val_usd == pytest.approx(2_255_873.6)
    assert row.pct_val == pytest.approx(0.0023838966190458206)
    assert row.payoff_profile == "Long"
    assert row.inv_country == "CN"
    assert row.is_restricted_sec == "N"
    assert row.fair_val_level == "2"
    assert row.is_loan_by_fund == "N"


def test_fund_disclosures_without_cik(client: Any, fixture_server: FixtureServer) -> None:
    """``disclosures`` omits ``cik`` when it is not given and keeps ``cik`` keyword-only."""
    fixture_server.route("/funds/disclosure", load_fixture("fund_disclosures.json"))
    rows = client.funds.disclosures("VWO", 2023, 4)

    assert fixture_server.requests[0].target == "/funds/disclosure?symbol=VWO&year=2023&quarter=4"
    assert fixture_server.requests[0].query == {"symbol": ["VWO"], "year": ["2023"], "quarter": ["4"]}
    assert len(rows) == 1
    with pytest.raises(TypeError):
        client.funds.disclosures("VWO", 2023, 4, "0000857489")
    with pytest.raises(TypeError):
        client.funds.disclosures("VWO", 2023)


def test_search_fund_disclosure_holders_encodes_punctuation(client: Any, fixture_server: FixtureServer) -> None:
    """``search_disclosure_holders`` form-encodes the exact name and decodes the string row."""
    fixture_server.route("/funds/disclosure-holders-search", load_fixture("fund_disclosure_holder_search.json"))
    rows = client.funds.search_disclosure_holders(FEDERATED)

    assert (
        fixture_server.requests[0].target
        == "/funds/disclosure-holders-search?name=Federated+Hermes+Government+Income+Securities%2C+Inc."
    )
    assert fixture_server.requests[0].query == {"name": [FEDERATED]}
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, FundDisclosureSearchResult)
    assert row.symbol == "FGOAX"
    assert row.cik == "0000355691"
    assert row.class_id == "C000024574"
    assert row.series_id == "S000009042"
    assert row.entity_name == FEDERATED
    assert row.entity_org_type == "30"
    assert row.class_name == "Class A Shares"
    assert row.reporting_file_number == "811-03266"
    assert row.address == "4000 ERICSSON DRIVE"
    assert row.zip_code == "15086-7561"
    assert row.state == "PA"


def test_disclosure_rows_decode_omitted_members_as_none(client: Any, fixture_server: FixtureServer) -> None:
    """A null ``symbol``, an empty ``isin``, and a null ``address`` decode as ``None``."""
    disclosure = load_fixture("fund_disclosures.json")
    disclosure[0].update(symbol=None, isin="")
    fixture_server.route("/funds/disclosure", disclosure)
    search = load_fixture("fund_disclosure_holder_search.json")
    search[0]["address"] = None
    fixture_server.route("/funds/disclosure-holders-search", search)

    position = client.funds.disclosures("VWO", 2023, 4)[0]
    assert position.symbol is None
    assert position.isin is None
    assert client.funds.search_disclosure_holders(FEDERATED)[0].address is None


def test_search_fund_disclosure_holders_with_a_plain_name(client: Any, fixture_server: FixtureServer) -> None:
    """``search_disclosure_holders`` sends a space-separated name as ``+`` and nothing else."""
    fixture_server.route("/funds/disclosure-holders-search", load_fixture("fund_disclosure_holder_search.json"))
    rows = client.funds.search_disclosure_holders("Vanguard Total World")

    assert fixture_server.requests[0].target == "/funds/disclosure-holders-search?name=Vanguard+Total+World"
    assert len(rows) == 1


def test_fund_disclosure_dates_with_cik(client: Any, fixture_server: FixtureServer) -> None:
    """``disclosure_dates`` encodes ``symbol`` then ``cik`` and returns the shared filing-date row."""
    fixture_server.route("/funds/disclosure-dates", load_fixture("fund_disclosure_dates.json"))
    rows = client.funds.disclosure_dates("VWO", cik="0000036405")

    assert fixture_server.requests[0].target == "/funds/disclosure-dates?symbol=VWO&cik=0000036405"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, Form13fFilingDate)
    assert row.date == datetime.date(2026, 4, 30)
    assert row.year == 2026
    assert row.quarter == 2


def test_fund_disclosure_dates_without_cik(client: Any, fixture_server: FixtureServer) -> None:
    """``disclosure_dates`` sends only ``symbol`` when ``cik`` is omitted."""
    fixture_server.route("/funds/disclosure-dates", load_fixture("fund_disclosure_dates.json"))
    rows = client.funds.disclosure_dates("VWO")

    assert fixture_server.requests[0].target == "/funds/disclosure-dates?symbol=VWO"
    assert len(rows) == 1
    assert isinstance(rows[0], Form13fFilingDate)
    with pytest.raises(TypeError):
        client.funds.disclosure_dates("VWO", "0000036405")


def test_disclosure_queries_decode_an_empty_array(client: Any, fixture_server: FixtureServer) -> None:
    """A documented empty body decodes to an empty list on the three disclosure queries."""
    fixture_server.route("/funds/disclosure", [])
    fixture_server.route("/funds/disclosure-holders-search", [])
    fixture_server.route("/funds/disclosure-dates", [])

    assert client.funds.disclosures("VWO", 2023, 4) == []
    assert client.funds.search_disclosure_holders("Vanguard") == []
    assert client.funds.disclosure_dates("VWO") == []
    assert [request.target for request in fixture_server.requests] == [
        "/funds/disclosure?symbol=VWO&year=2023&quarter=4",
        "/funds/disclosure-holders-search?name=Vanguard",
        "/funds/disclosure-dates?symbol=VWO",
    ]


@pytest.mark.parametrize(
    ("method", "args"),
    [
        pytest.param("etf_holdings", (" ",), id="etf_holdings"),
        pytest.param("etf_info", ("",), id="etf_info"),
        pytest.param("etf_country_weightings", ("\t",), id="etf_country_weightings"),
        pytest.param("etf_asset_exposure", ("  ",), id="etf_asset_exposure"),
        pytest.param("etf_sector_weightings", ("",), id="etf_sector_weightings"),
        pytest.param("latest_disclosure_holders", (" ",), id="latest_disclosure_holders"),
        pytest.param("disclosures", ("", 2023, 4), id="disclosures"),
        pytest.param("disclosure_dates", (" ",), id="disclosure_dates"),
    ],
)
def test_blank_symbol_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, args: tuple[Any, ...]
) -> None:
    """A blank ticker is rejected locally on every ticker-taking method with ``symbol`` as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.funds, method)(*args)
    error = raised.value
    assert str(error) == "symbol: value must not be empty or whitespace-only"
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize("year", [-1, 4_294_967_296])
def test_out_of_range_year_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, year: int
) -> None:
    """A year outside the provider's unsigned 32-bit range is rejected locally."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.funds.disclosures("VWO", year, 4)
    assert str(raised.value) == "year: must be an integer from 0 through 4294967295"
    assert fixture_server.requests == []


@pytest.mark.parametrize("quarter", [0, 5, -1])
def test_out_of_range_quarter_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, quarter: int
) -> None:
    """A quarter outside 1 through 4 is rejected locally; a float quarter is a ``TypeError``."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.funds.disclosures("VWO", 2023, quarter)
    assert str(raised.value) == "quarter: quarter must be an integer from 1 through 4"
    assert fixture_server.requests == []
    with pytest.raises(TypeError):
        client.funds.disclosures("VWO", 2023, 4.0)


@pytest.mark.parametrize(
    ("method", "args"),
    [
        pytest.param("disclosures", ("VWO", 2023, 4), id="disclosures"),
        pytest.param("disclosure_dates", ("VWO",), id="disclosure_dates"),
    ],
)
def test_blank_cik_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, args: tuple[Any, ...]
) -> None:
    """A blank optional ``cik`` is rejected locally instead of being sent as an empty parameter."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.funds, method)(*args, cik="  ")
    error = raised.value
    assert str(error) == "cik: value must not be empty or whitespace-only"
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("name", "message"),
    [
        pytest.param("", "name: value must not be empty or whitespace-only", id="empty"),
        pytest.param("   ", "name: value must not be empty or whitespace-only", id="whitespace"),
        pytest.param("Vanguard\tTotal", "name: value must not contain control characters", id="control-char"),
    ],
)
def test_invalid_search_name_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, name: str, message: str
) -> None:
    """``search_term`` validation fails locally and names ``name``."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.funds.search_disclosure_holders(name)
    error = raised.value
    assert str(error) == message
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_status_error_carries_the_funds_endpoint_id(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on a nested funds path maps to ``FmpStatusError`` with the endpoint id."""
    fixture_server.route("/funds/disclosure", {"error": "denied"}, status=401)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.funds.disclosures("VWO", 2023, 4)
    error = raised.value
    assert error.endpoint == "funds/disclosure"
    assert error.status == 401
    assert error.body == '{"error": "denied"}'


def test_non_json_body_is_a_decode_error(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-JSON body on the ETF path maps to ``FmpDecodeError``."""
    fixture_server.route("/etf/holdings", b"not-json")
    with pytest.raises(errors.FmpDecodeError):
        client.funds.etf_holdings("SPY")


def test_wrong_percent_kind_is_a_decode_error(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A numeric country weight where the provider documents a percent string is a decode failure."""
    body = load_fixture("etf_country_weightings.json")
    body[0]["weightPercentage"] = 97.26
    fixture_server.route("/etf/country-weightings", body)
    with pytest.raises(errors.FmpDecodeError):
        client.funds.etf_country_weightings("SPY")


def test_fund_disclosures_decode_a_negative_fractional_balance(client: Any, fixture_server: FixtureServer) -> None:
    """Issue #340: a short position's fractional, negative balance decodes as ``float``."""
    fixture_server.route("/funds/disclosure", load_fixture("fund_disclosures_fractional_synthetic.json"))
    rows = client.funds.disclosures("VWO", 2023, 4)

    assert len(rows) == 1
    assert isinstance(rows[0].balance, float)
    assert rows[0].balance == -2_438_784.5
