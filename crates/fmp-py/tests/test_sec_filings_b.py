"""Runtime contract of ``client.sec_filings`` for lookups and classifications.

Covers the three company searches, ``company_profile``, the two typed
industry-classification methods, and the negative cases: one test per method
routes the documented fixture body, calls the method with one argument shape,
and asserts the exact request target plus a few typed fields. The expected
targets are the ones the Rust ``sec_company_lookup_endpoints.rs`` and
``sec_industry_classification_endpoints.rs`` tests pin. The filing feeds live
in ``test_sec_filings_a.py``.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.sec_filings import SecCompanyProfile, SecCompanySearchResult, SicClassification

FROM_2024_01_01 = datetime.date(2024, 1, 1)
TO_2024_03_01 = datetime.date(2024, 3, 1)
U32_MAX = 4_294_967_295


def test_search_companies_by_name_encodes_punctuation(client: Any, fixture_server: FixtureServer) -> None:
    """``search_companies_by_name`` sends the text under ``company`` and keeps the ``None`` symbol."""
    fixture_server.route("/sec-filings-company-search/name", load_fixture("sec_companies_by_name.json"))
    rows = client.sec_filings.search_companies_by_name("Berkshire, Hathaway / Fund")

    assert fixture_server.requests[0].target == "/sec-filings-company-search/name?company=Berkshire%2C+Hathaway+%2F+Fund"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SecCompanySearchResult)
    assert row.symbol == "None"
    assert row.name == "BERKSHIRE MULTIFAMILY VALUE FUND II LP"
    assert row.cik == "0001418405"
    assert (row.sic_code, row.industry_title) == ("", "")
    assert row.business_address == "c/o Berkshire Property Advisors LLC, Boston MA 02108"
    assert row.phone_number == "(617) 646-2300"


def test_search_companies_by_symbol_encodes_a_spaced_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``search_companies_by_symbol`` form-encodes the ticker and takes no other argument."""
    fixture_server.route("/sec-filings-company-search/symbol", load_fixture("sec_companies_by_symbol.json"))
    rows = client.sec_filings.search_companies_by_symbol("BRK.B / Class A")

    assert fixture_server.requests[0].target == "/sec-filings-company-search/symbol?symbol=BRK.B+%2F+Class+A"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SecCompanySearchResult)
    assert row.symbol == "AAPL"
    assert row.cik == "0000320193"
    assert row.sic_code == "3571"
    assert row.industry_title == "ELECTRONIC COMPUTERS"
    with pytest.raises(TypeError):
        client.sec_filings.search_companies_by_symbol("AAPL", limit=5)


def test_search_companies_by_cik_keeps_leading_zeroes(client: Any, fixture_server: FixtureServer) -> None:
    """``search_companies_by_cik`` sends the CIK verbatim under ``cik``."""
    fixture_server.route("/sec-filings-company-search/cik", load_fixture("sec_companies_by_cik.json"))
    rows = client.sec_filings.search_companies_by_cik("0000320193")

    assert fixture_server.requests[0].target == "/sec-filings-company-search/cik?cik=0000320193"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SecCompanySearchResult)
    assert row.name == "APPLE INC."
    assert row.business_address == "ONE APPLE PARK WAY, CUPERTINO CA 95014"
    assert row.phone_number == "(408) 996-1010"


def test_company_profile_with_cik_a(client: Any, fixture_server: FixtureServer) -> None:
    """``company_profile`` encodes the literal ``cik-A`` key after ``symbol``."""
    fixture_server.route("/sec-profile", load_fixture("sec_company_profile.json"))
    rows = client.sec_filings.company_profile("AAPL", cik_a="0000320193")

    assert fixture_server.requests[0].target == "/sec-profile?symbol=AAPL&cik-A=0000320193"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SecCompanyProfile)
    assert row.symbol == "AAPL"
    assert row.cik == "0000320193"
    assert row.isin == "US0378331005"
    assert row.ipo_date == datetime.date(1980, 12, 12)
    assert row.employees == "166000"
    assert row.exchange == "NASDAQ"
    assert row.fiscal_year_end == "09-30"
    assert row.fifty_two_week_range == "201.5 - 344.57"
    assert row.market_sector == "Technology"
    assert row.open_figi_composite == "BBG000B9XRY4"
    assert row.security_type is None
    assert row.is_active is True
    assert (row.is_etf, row.is_adr, row.is_fund) == (False, False, False)


def test_company_profile_without_cik_a(client: Any, fixture_server: FixtureServer) -> None:
    """``company_profile`` sends only the symbol when ``cik_a`` is not given."""
    fixture_server.route("/sec-profile", load_fixture("sec_company_profile.json"))
    rows = client.sec_filings.company_profile("AAPL")

    assert fixture_server.requests[0].target == "/sec-profile?symbol=AAPL"
    assert len(rows) == 1
    assert rows[0].ceo == "Timothy D. Cook"
    assert rows[0].country == "US"


def test_industry_classifications_with_both_filters(client: Any, fixture_server: FixtureServer) -> None:
    """``industry_classifications`` encodes ``industryTitle`` then ``sicCode`` as free text."""
    fixture_server.route("/standard-industrial-classification-list", load_fixture("industry_classifications.json"))
    rows = client.sec_filings.industry_classifications(industry_title="SERVICES, NEC / OTHER", sic_code="07371")

    assert (
        fixture_server.requests[0].target
        == "/standard-industrial-classification-list?industryTitle=SERVICES%2C+NEC+%2F+OTHER&sicCode=07371"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SicClassification)
    assert row.office == "Office of Life Sciences"
    assert row.sic_code == "100"
    assert row.industry_title == "AGRICULTURAL PRODUCTION-CROPS"


def test_industry_classifications_without_filters(client: Any, fixture_server: FixtureServer) -> None:
    """``industry_classifications`` sends a bare path when no filter is given."""
    fixture_server.route("/standard-industrial-classification-list", load_fixture("industry_classifications.json"))
    rows = client.sec_filings.industry_classifications()

    assert fixture_server.requests[0].target == "/standard-industrial-classification-list"
    assert len(rows) == 1
    with pytest.raises(TypeError):
        client.sec_filings.industry_classifications("07371")


def test_industry_classifications_with_sic_code_only(client: Any, fixture_server: FixtureServer) -> None:
    """``industry_classifications`` keeps the two filters independent."""
    fixture_server.route("/standard-industrial-classification-list", load_fixture("industry_classifications.json"))
    client.sec_filings.industry_classifications(sic_code="07371")

    assert fixture_server.requests[0].target == "/standard-industrial-classification-list?sicCode=07371"


def test_all_industry_classifications_with_pagination(client: Any, fixture_server: FixtureServer) -> None:
    """``all_industry_classifications`` encodes ``page`` then ``limit`` and shares the search row type."""
    fixture_server.route("/all-industry-classification", load_fixture("all_industry_classifications.json"))
    rows = client.sec_filings.all_industry_classifications(page=0, limit=U32_MAX)

    assert fixture_server.requests[0].target == "/all-industry-classification?page=0&limit=4294967295"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, SecCompanySearchResult)
    assert row.symbol == "0Q16.L"
    assert row.cik == "0000070858"
    assert row.sic_code == "6021"
    assert row.industry_title == "NATIONAL COMMERCIAL BANKS"
    assert row.business_address == "['BANK OF AMERICA CORPORATE CENTER', 'CHARLOTTE NC 28255']"


def test_all_industry_classifications_without_options(client: Any, fixture_server: FixtureServer) -> None:
    """``all_industry_classifications`` sends a bare path when no pagination is given."""
    fixture_server.route("/all-industry-classification", load_fixture("all_industry_classifications.json"))
    rows = client.sec_filings.all_industry_classifications()

    assert fixture_server.requests[0].target == "/all-industry-classification"
    assert len(rows) == 1


@pytest.mark.parametrize(
    ("keyword", "value"),
    [("from_", "01/01/2024"), ("to", "2024-13-01"), ("from_", ""), ("to", "2024-3-1")],
)
def test_non_iso_date_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, keyword: str, value: str
) -> None:
    """A date string that is not ``YYYY-MM-DD`` fails locally, prefixed with the keyword."""
    dates = {"from_": "2024-01-01", "to": "2024-03-01", keyword: value}
    with pytest.raises(errors.FmpValidationError) as raised:
        client.sec_filings.latest_8k(**dates)
    error = raised.value
    assert str(error) == f"{keyword}: value must be a valid YYYY-MM-DD date"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_negative_page_names_the_argument(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A negative ``page`` is rejected locally with the keyword as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.sec_filings.all_industry_classifications(page=-1)
    assert str(raised.value) == "page: must be an integer from 0 through 4294967295"
    assert fixture_server.requests == []


def test_negative_limit_names_the_argument(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A negative ``limit`` is rejected locally with the keyword as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.sec_filings.by_symbol("AAPL", FROM_2024_01_01, TO_2024_03_01, limit=-1)
    assert str(raised.value).startswith("limit: ")
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("method", "args", "keyword"),
    [
        ("by_form_type", ("  ", FROM_2024_01_01, TO_2024_03_01), "form_type"),
        ("by_cik", ("", FROM_2024_01_01, TO_2024_03_01), "cik"),
        ("search_companies_by_name", ("  ",), "company"),
        ("search_companies_by_symbol", ("",), "symbol"),
    ],
)
def test_blank_identifier_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, args: tuple[Any, ...], keyword: str
) -> None:
    """A blank string-backed identifier is rejected locally with the keyword as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.sec_filings, method)(*args)
    assert str(raised.value) == f"{keyword}: value must not be empty or whitespace-only"
    assert fixture_server.requests == []


def test_blank_optional_cik_a_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """An optional identifier setter validates its value under its own keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.sec_filings.company_profile("AAPL", cik_a=" ")
    assert str(raised.value).startswith("cik_a: ")
    assert fixture_server.requests == []


def test_status_error_carries_the_sec_endpoint_id(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on a nested path maps to ``FmpStatusError`` with the endpoint id."""
    fixture_server.route("/sec-filings-search/cik", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.sec_filings.by_cik("0000320193", FROM_2024_01_01, TO_2024_03_01)
    error = raised.value
    assert error.endpoint == "sec-filings-search/cik"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_non_json_body_is_a_decode_error(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-JSON body on a lookup maps to ``FmpDecodeError``."""
    fixture_server.route("/sec-profile", b"not-json")
    with pytest.raises(errors.FmpDecodeError):
        client.sec_filings.company_profile("AAPL")
