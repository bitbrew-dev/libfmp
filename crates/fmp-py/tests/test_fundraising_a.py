"""Runtime contract of ``client.fundraising``: crowdfunding and Regulation D offerings.

One test per method routes the documented fixture body, calls the method with
one argument shape, and asserts the exact request target plus a few typed
fields (including the ``datetime.date`` / ``datetime.datetime`` ones). The
expected targets are the ones the Rust ``fundraising_crowdfunding_endpoints.rs``,
``fundraising_regulation_d_endpoints.rs``, and ``fundraising_search_endpoints.rs``
tests pin. The negative cases and the error mapping live in
``test_fundraising_b.py``.
"""

import datetime
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.fundraising import (
    CrowdfundingOffering,
    CrowdfundingOfferingSearchResult,
    FundraisingNamespace,
    RegulationDOffering,
    RegulationDOfferingSearchResult,
)

OYO_FITNESS_CIK = "0001916078"
NJOY_CIK = "0001547416"
GIP_CIK = "0002013736"
SPACED_NAME = "NJOY / Class A"
SPACED_NAME_ENCODED = "NJOY+%2F+Class+A"
U32_MAX = 4_294_967_295


def test_fundraising_namespace_is_the_generated_type(client: Any) -> None:
    """``client.fundraising`` is the generated flat namespace class."""
    assert isinstance(client.fundraising, FundraisingNamespace)


def test_latest_crowdfunding_offerings_with_page_and_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_crowdfunding_offerings`` encodes ``page`` before ``limit`` and decodes the US-dated row."""
    fixture_server.route("/crowdfunding-offerings-latest", load_fixture("crowdfunding_offerings_latest.json"))
    rows = client.fundraising.latest_crowdfunding_offerings(page=0, limit=100)

    assert fixture_server.requests[0].target == "/crowdfunding-offerings-latest?page=0&limit=100"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CrowdfundingOffering)
    assert row.cik == "0001621902"
    assert row.company_name == "Cutting Edge Superconductors, Inc."
    assert row.form_type == "C/A"
    assert row.date == datetime.date(2011, 11, 22)
    assert row.filing_date == datetime.datetime(2026, 7, 30, 0, 0, 0)
    assert row.accepted_date == datetime.datetime(2026, 7, 30, 12, 54, 38)
    assert row.offering_deadline_date == datetime.date(2026, 10, 31)
    assert row.intermediary_commission_cik == "0001669191"
    assert row.number_of_security_offered == 100_000
    assert row.offering_amount == 10_000
    assert isinstance(row.offering_price, float)
    assert row.offering_price == pytest.approx(0.1)
    assert row.over_subscription_accepted == "Y"
    assert row.security_offered_other_description is None
    assert row.net_income_most_recent_fiscal_year == -152_577


def test_latest_crowdfunding_offerings_with_limit_only(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_crowdfunding_offerings`` with only ``limit`` omits ``page`` and accepts a zero limit."""
    fixture_server.route("/crowdfunding-offerings-latest", load_fixture("crowdfunding_offerings_latest.json"))
    rows = client.fundraising.latest_crowdfunding_offerings(limit=0)

    assert fixture_server.requests[0].target == "/crowdfunding-offerings-latest?limit=0"
    assert len(rows) == 1


def test_latest_crowdfunding_offerings_without_options_sends_the_bare_path(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``latest_crowdfunding_offerings`` with no options requests the bare path and takes no positional argument."""
    fixture_server.route("/crowdfunding-offerings-latest", load_fixture("crowdfunding_offerings_latest.json"))
    rows = client.fundraising.latest_crowdfunding_offerings()

    assert fixture_server.requests[0].target == "/crowdfunding-offerings-latest"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1
    with pytest.raises(TypeError):
        client.fundraising.latest_crowdfunding_offerings(0)


def test_crowdfunding_offerings_by_cik_keeps_leading_zeroes(client: Any, fixture_server: FixtureServer) -> None:
    """``crowdfunding_offerings_by_cik`` sends the CIK verbatim and decodes the integer-priced row."""
    fixture_server.route("/crowdfunding-offerings", load_fixture("crowdfunding_offerings_by_cik.json"))
    rows = client.fundraising.crowdfunding_offerings_by_cik(OYO_FITNESS_CIK)

    assert fixture_server.requests[0].target == f"/crowdfunding-offerings?cik={OYO_FITNESS_CIK}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CrowdfundingOffering)
    assert row.cik == OYO_FITNESS_CIK
    assert row.company_name == "OYO Fitness, Inc"
    assert row.form_type == "C-U"
    assert row.form_signification == "Progress Update"
    assert row.date == datetime.date(2021, 12, 31)
    assert row.filing_date == datetime.datetime(2022, 7, 21, 0, 0, 0)
    assert row.accepted_date == datetime.datetime(2022, 7, 21, 17, 28, 54)
    assert row.offering_deadline_date == datetime.date(2022, 7, 19)
    assert row.intermediary_company_name == "StartEngine Capital, LLC"
    assert isinstance(row.offering_price, int)
    assert row.offering_price == 2
    assert row.maximum_offering_amount == 1_070_000
    assert row.current_number_of_employees == 5


def test_crowdfunding_offerings_by_cik_accepts_the_keyword(client: Any, fixture_server: FixtureServer) -> None:
    """``crowdfunding_offerings_by_cik`` takes ``cik`` by keyword and rejects extra options."""
    fixture_server.route("/crowdfunding-offerings", load_fixture("crowdfunding_offerings_by_cik.json"))
    rows = client.fundraising.crowdfunding_offerings_by_cik(cik=OYO_FITNESS_CIK)

    assert fixture_server.requests[0].query == {"cik": [OYO_FITNESS_CIK]}
    assert len(rows) == 1
    with pytest.raises(TypeError):
        client.fundraising.crowdfunding_offerings_by_cik(OYO_FITNESS_CIK, limit=5)


def test_search_crowdfunding_offerings_encodes_punctuation(client: Any, fixture_server: FixtureServer) -> None:
    """``search_crowdfunding_offerings`` form-encodes the text under ``name`` and keeps the null date."""
    fixture_server.route("/crowdfunding-offerings-search", load_fixture("crowdfunding_offerings_search.json"))
    rows = client.fundraising.search_crowdfunding_offerings(SPACED_NAME)

    assert fixture_server.requests[0].target == f"/crowdfunding-offerings-search?name={SPACED_NAME_ENCODED}"
    assert fixture_server.requests[0].query["name"] == [SPACED_NAME]
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CrowdfundingOfferingSearchResult)
    assert row.cik == "0001912939"
    assert row.name == "Enotap LLC"
    assert row.date is None


def test_search_crowdfunding_offerings_sends_plain_text_verbatim(client: Any, fixture_server: FixtureServer) -> None:
    """``search_crowdfunding_offerings`` sends a plain search term unchanged and takes it by keyword."""
    fixture_server.route("/crowdfunding-offerings-search", load_fixture("crowdfunding_offerings_search.json"))
    rows = client.fundraising.search_crowdfunding_offerings(name="enotap")

    assert fixture_server.requests[0].target == "/crowdfunding-offerings-search?name=enotap"
    assert len(rows) == 1


def test_search_regulation_d_offerings_encodes_punctuation(client: Any, fixture_server: FixtureServer) -> None:
    """``search_regulation_d_offerings`` maps to ``/fundraising-search`` and decodes the datetime field."""
    fixture_server.route("/fundraising-search", load_fixture("fundraising_search.json"))
    rows = client.fundraising.search_regulation_d_offerings(SPACED_NAME)

    assert fixture_server.requests[0].target == f"/fundraising-search?name={SPACED_NAME_ENCODED}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, RegulationDOfferingSearchResult)
    assert row.cik == NJOY_CIK
    assert row.name == "NJOY INC"
    assert row.date == datetime.datetime(2014, 2, 28, 16, 0, 25)


def test_search_regulation_d_offerings_sends_plain_text_verbatim(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``search_regulation_d_offerings`` sends a plain search term unchanged."""
    fixture_server.route("/fundraising-search", load_fixture("fundraising_search.json"))
    rows = client.fundraising.search_regulation_d_offerings("NJOY")

    assert fixture_server.requests[0].target == "/fundraising-search?name=NJOY"
    assert len(rows) == 1


def test_latest_regulation_d_offerings_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_regulation_d_offerings`` encodes ``page``, ``limit``, then ``cik`` and decodes the row."""
    fixture_server.route("/fundraising-latest", load_fixture("fundraising_latest.json"))
    rows = client.fundraising.latest_regulation_d_offerings(page=0, limit=10, cik=GIP_CIK)

    assert fixture_server.requests[0].target == f"/fundraising-latest?page=0&limit=10&cik={GIP_CIK}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, RegulationDOffering)
    assert row.cik == "0002127786"
    assert row.company_name == "GIP V Horizon Co-Invest 1, L.P."
    assert row.entity_name == "GIP V Horizon Co-Invest 1, L.P."
    assert row.entity_type == "Limited Partnership"
    assert row.form_type == "D"
    assert row.date == datetime.date(2026, 7, 30)
    assert row.filing_date == datetime.datetime(2026, 7, 30, 0, 0, 0)
    assert row.accepted_date == datetime.datetime(2026, 7, 30, 13, 5, 23)
    assert row.date_of_first_sale is None
    assert row.incorporated_within_five_years is True
    assert row.year_of_incorporation == "2026"
    assert row.federal_exemptions_exclusions == "06b, 3C, 3C.7"
    assert row.is_amendment is False
    assert row.securities_offered_are_of_equity_type is True
    assert row.total_amount_sold == 0
    assert row.total_number_already_invested == 0


def test_latest_regulation_d_offerings_with_cik_only(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_regulation_d_offerings`` with only ``cik`` omits both pagination keys."""
    fixture_server.route("/fundraising-latest", load_fixture("fundraising_latest.json"))
    rows = client.fundraising.latest_regulation_d_offerings(cik=GIP_CIK)

    assert fixture_server.requests[0].target == f"/fundraising-latest?cik={GIP_CIK}"
    assert len(rows) == 1


def test_latest_regulation_d_offerings_accepts_the_u32_boundary(client: Any, fixture_server: FixtureServer) -> None:
    """``latest_regulation_d_offerings`` passes ``2**32 - 1`` for ``page`` and a zero ``limit`` unchanged."""
    fixture_server.route("/fundraising-latest", load_fixture("fundraising_latest.json"))
    rows = client.fundraising.latest_regulation_d_offerings(page=U32_MAX, limit=0)

    assert fixture_server.requests[0].target == f"/fundraising-latest?page={U32_MAX}&limit=0"
    assert len(rows) == 1


def test_latest_regulation_d_offerings_without_options_sends_the_bare_path(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``latest_regulation_d_offerings`` with no options requests the bare path and takes no positional argument."""
    fixture_server.route("/fundraising-latest", load_fixture("fundraising_latest.json"))
    rows = client.fundraising.latest_regulation_d_offerings()

    assert fixture_server.requests[0].target == "/fundraising-latest"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1
    with pytest.raises(TypeError):
        client.fundraising.latest_regulation_d_offerings(GIP_CIK)


def test_regulation_d_offerings_by_cik_keeps_leading_zeroes(client: Any, fixture_server: FixtureServer) -> None:
    """``regulation_d_offerings_by_cik`` maps to the bare ``/fundraising`` path and decodes the dated row."""
    fixture_server.route("/fundraising", load_fixture("fundraising_by_cik.json"))
    rows = client.fundraising.regulation_d_offerings_by_cik(NJOY_CIK)

    assert fixture_server.requests[0].target == f"/fundraising?cik={NJOY_CIK}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, RegulationDOffering)
    assert row.cik == NJOY_CIK
    assert row.company_name == "NJOY INC"
    assert row.form_signification == "Notice of Exempt Offering of Securities"
    assert row.date == datetime.date(2014, 2, 28)
    assert row.filing_date == datetime.datetime(2014, 2, 28, 0, 0, 0)
    assert row.accepted_date == datetime.datetime(2014, 2, 28, 16, 0, 25)
    assert row.date_of_first_sale == datetime.date(2014, 2, 14)
    assert row.incorporated_within_five_years is None
    assert row.year_of_incorporation == ""
    assert row.related_person_relationship == "Executive Officer, Director"
    assert row.total_offering_amount == 71_999_990
    assert row.total_amount_sold == 71_999_990
    assert row.total_number_already_invested == 24
    assert row.has_non_accredited_investors is False


def test_regulation_d_offerings_by_cik_accepts_the_keyword(client: Any, fixture_server: FixtureServer) -> None:
    """``regulation_d_offerings_by_cik`` takes ``cik`` by keyword and rejects extra options."""
    fixture_server.route("/fundraising", load_fixture("fundraising_by_cik.json"))
    rows = client.fundraising.regulation_d_offerings_by_cik(cik=NJOY_CIK)

    assert fixture_server.requests[0].query == {"cik": [NJOY_CIK]}
    assert len(rows) == 1
    with pytest.raises(TypeError):
        client.fundraising.regulation_d_offerings_by_cik(NJOY_CIK, page=0)


def test_regulation_d_offerings_decode_fractional_amounts(client: Any, fixture_server: FixtureServer) -> None:
    """Issue #340: fractional and integral-float offering amounts decode as ``float``."""
    fixture_server.route("/fundraising", load_fixture("fundraising_by_cik_fractional_synthetic.json"))
    rows = client.fundraising.regulation_d_offerings_by_cik(NJOY_CIK)

    assert len(rows) == 1
    assert isinstance(rows[0].total_offering_amount, float)
    assert rows[0].total_offering_amount == 71_999_990.5
    assert rows[0].total_amount_sold == 1.0


def test_latest_crowdfunding_offerings_decode_null_members_to_none(client: Any, fixture_server: FixtureServer) -> None:
    """The members FMP sends as null on older filings decode to ``None``."""
    body = load_fixture("crowdfunding_offerings_latest.json")
    body[0]["compensationAmount"] = None
    body[0]["financialInterest"] = None
    body[0]["intermediaryCommissionFileNumber"] = None
    body[0]["intermediaryCompanyName"] = None
    body[0]["issuerWebsite"] = None
    body[0]["offeringDeadlineDate"] = None
    body[0]["overSubscriptionAllocationType"] = None
    body[0]["securityOfferedType"] = None
    fixture_server.route("/crowdfunding-offerings-latest", body)
    rows = client.fundraising.latest_crowdfunding_offerings()

    assert rows[0].compensation_amount is None
    assert rows[0].financial_interest is None
    assert rows[0].intermediary_commission_file_number is None
    assert rows[0].intermediary_company_name is None
    assert rows[0].issuer_website is None
    assert rows[0].offering_deadline_date is None
    assert rows[0].over_subscription_allocation_type is None
    assert rows[0].security_offered_type is None


def test_latest_regulation_d_offerings_decode_null_members_to_none(client: Any, fixture_server: FixtureServer) -> None:
    """A null ``revenueRange`` or ``securitiesOfferedAreOfEquityType`` decodes to ``None``."""
    body = load_fixture("fundraising_latest.json")
    body[0]["revenueRange"] = None
    body[0]["securitiesOfferedAreOfEquityType"] = None
    fixture_server.route("/fundraising-latest", body)
    rows = client.fundraising.latest_regulation_d_offerings()

    assert rows[0].revenue_range is None
    assert rows[0].securities_offered_are_of_equity_type is None
