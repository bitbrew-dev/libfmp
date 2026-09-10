"""Runtime contract of ``client.congressional``: profiles, positions, net worth, negatives.

The expected targets are the ones the Rust ``congressional_member_endpoints.rs``
and ``congressional_net_worth_endpoints.rs`` tests pin. The eight chamber
trade feeds live in ``test_congressional_a.py``.
"""

import datetime
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.congressional import (
    CongressionalDebtDetails,
    CongressionalMemberNetWorthAggregate,
    CongressionalMemberNetWorthEntry,
    CongressionalMemberPosition,
    CongressionalMemberProfile,
    CongressionalNetWorthRange,
)

MEMBER_ID = "P000197"
PROFILE_WITH_EVERY_OPTION = (
    "/senate-profile?active=false&senateID=P000197&latestParty=Independent+%2F+Other"
    "&latestPosition=Representative+At-Large&page=0&limit=500"
)
POSITIONS_WITH_EVERY_OPTION = (
    "/senate-positions?senateID=P000197&party=Republican+%2F+Other&position=Representative+At-Large&page=0&limit=300"
)


def test_profiles_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``profiles`` encodes ``active``, ``senateID``, the two text filters, ``page``, then ``limit``."""
    fixture_server.route("/senate-profile", load_fixture("congress_senate_profile.json"))
    rows = client.congressional.profiles(
        active=False,
        member_id=MEMBER_ID,
        latest_party="Independent / Other",
        latest_position="Representative At-Large",
        page=0,
        limit=500,
    )

    assert fixture_server.requests[0].target == PROFILE_WITH_EVERY_OPTION
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CongressionalMemberProfile)
    assert row.member_id == "L000397"
    assert row.first_name == "Zoe"
    assert row.last_name == "Lofgren"
    assert row.birth_date == datetime.date(1947, 12, 20)
    assert row.latest_party == "Democrat"
    assert row.latest_state == "CA"
    assert row.latest_position == "Representative"
    assert row.image == "https://images.financialmodelingprep.com/senate/L000397.jpg"
    assert row.active is True
    assert row.years_active == pytest.approx(31.6)


def test_profiles_with_active_true_only(client: Any, fixture_server: FixtureServer) -> None:
    """``active=True`` is sent as lowercase ``true`` and nothing else is encoded."""
    fixture_server.route("/senate-profile", load_fixture("congress_senate_profile.json"))
    rows = client.congressional.profiles(active=True)

    assert fixture_server.requests[0].target == "/senate-profile?active=true"
    assert fixture_server.requests[0].query == {"active": ["true"]}
    assert len(rows) == 1


def test_profiles_without_options_sends_the_bare_path(client: Any, fixture_server: FixtureServer) -> None:
    """``profiles`` with nothing set requests the bare ``/senate-profile``."""
    fixture_server.route("/senate-profile", load_fixture("congress_senate_profile.json"))
    rows = client.congressional.profiles()

    assert fixture_server.requests[0].target == "/senate-profile"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1


def test_positions_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``positions`` encodes ``senateID``, ``party``, ``position``, ``page``, then ``limit``."""
    fixture_server.route("/senate-positions", load_fixture("congress_senate_positions.json"))
    rows = client.congressional.positions(
        member_id=MEMBER_ID,
        party="Republican / Other",
        position="Representative At-Large",
        page=0,
        limit=300,
    )

    assert fixture_server.requests[0].target == POSITIONS_WITH_EVERY_OPTION
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CongressionalMemberPosition)
    assert row.member_id == "Z000018"
    assert row.congress_number == 119
    assert row.start_date == datetime.date(2025, 1, 2)
    assert row.end_date is None
    assert row.party == "Republican"
    assert row.position == "Representative"
    assert row.state == "MT"
    assert row.years_in_term == pytest.approx(0.7)


def test_positions_with_party_only(client: Any, fixture_server: FixtureServer) -> None:
    """The text filters are independently optional; ``party`` is sent alone."""
    fixture_server.route("/senate-positions", load_fixture("congress_senate_positions.json"))
    rows = client.congressional.positions(party="Independent")

    assert fixture_server.requests[0].target == "/senate-positions?party=Independent"
    assert len(rows) == 1


def test_positions_without_options_sends_the_bare_path(client: Any, fixture_server: FixtureServer) -> None:
    """``positions`` with nothing set requests the bare ``/senate-positions``."""
    fixture_server.route("/senate-positions", load_fixture("congress_senate_positions.json"))
    rows = client.congressional.positions()

    assert fixture_server.requests[0].target == "/senate-positions"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1


def test_net_worth_with_page_and_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``net_worth`` encodes the required ``senateID``, then ``page``, then ``limit``."""
    fixture_server.route("/senate-net-worth", load_fixture("congress_senate_net_worth.json"))
    rows = client.congressional.net_worth(MEMBER_ID, page=0, limit=250)

    assert fixture_server.requests[0].target == "/senate-net-worth?senateID=P000197&page=0&limit=250"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CongressionalMemberNetWorthEntry)
    assert row.member_id == MEMBER_ID
    assert row.form_type == "House Report"
    assert row.year == 2022
    assert row.filing_date == datetime.date(2023, 5, 15)
    assert row.section == "Liabilities"
    assert row.category == "Mortgage & Real Estate Liability"
    assert row.name == "Union Bank of California"
    assert row.owner == "Joint"
    assert row.value == 3_000_001
    assert row.comment is None
    assert row.income_type is None
    assert row.income_range is None
    assert isinstance(row.value_range, CongressionalNetWorthRange)
    assert row.value_range.min == 1_000_001
    assert row.value_range.max == 5_000_000
    assert isinstance(row.debt_details, CongressionalDebtDetails)
    assert row.debt_details.date_incurred == "September 2007"


def test_net_worth_with_member_id_only(client: Any, fixture_server: FixtureServer) -> None:
    """``net_worth`` sends ``senateID`` alone when pagination is omitted."""
    fixture_server.route("/senate-net-worth", load_fixture("congress_senate_net_worth.json"))
    rows = client.congressional.net_worth(MEMBER_ID)

    assert fixture_server.requests[0].target == "/senate-net-worth?senateID=P000197"
    assert fixture_server.requests[0].query == {"senateID": [MEMBER_ID]}
    assert len(rows) == 1
    assert rows[0].link == "https://disclosures-clerk.house.gov/public_disc/financial-pdfs/2022/10053231.pdf"


def test_net_worth_aggregated_with_member_id_only(client: Any, fixture_server: FixtureServer) -> None:
    """``net_worth_aggregated`` sends ``senateID`` alone and decodes the yearly totals."""
    fixture_server.route("/senate-net-worth-aggregated", load_fixture("congress_senate_net_worth_aggregated.json"))
    rows = client.congressional.net_worth_aggregated(MEMBER_ID)

    assert fixture_server.requests[0].target == "/senate-net-worth-aggregated?senateID=P000197"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CongressionalMemberNetWorthAggregate)
    assert row.member_id == MEMBER_ID
    assert row.year == 2024
    assert row.total == 225_219_551
    assert row.stock == 136_748_525
    assert row.real_estate == 45_032_504
    assert row.business_and_self_employment == 0
    assert row.mutual_funds_and_etfs == 32_501
    assert row.revolving_and_credit_lines == 1_500_002


def test_net_worth_aggregated_with_totals_col(client: Any, fixture_server: FixtureServer) -> None:
    """``totals_col`` is sent verbatim as ``totalsCol`` after ``senateID``."""
    fixture_server.route("/senate-net-worth-aggregated", load_fixture("congress_senate_net_worth_aggregated.json"))
    rows = client.congressional.net_worth_aggregated(MEMBER_ID, totals_col="stock")

    assert fixture_server.requests[0].target == "/senate-net-worth-aggregated?senateID=P000197&totalsCol=stock"
    assert len(rows) == 1
    assert rows[0].total == 225_219_551


def test_required_member_id_cannot_be_omitted(client: Any, fixture_server: FixtureServer) -> None:
    """``net_worth`` and ``net_worth_aggregated`` take the member ID positionally and require it."""
    with pytest.raises(TypeError):
        client.congressional.net_worth()
    with pytest.raises(TypeError):
        client.congressional.net_worth_aggregated(totals_col="stock")
    assert fixture_server.requests == []


def test_optional_filters_are_keyword_only(client: Any, fixture_server: FixtureServer) -> None:
    """Every optional is keyword-only; a positional value is a ``TypeError``."""
    with pytest.raises(TypeError):
        client.congressional.profiles(True)
    with pytest.raises(TypeError):
        client.congressional.senate_trades("AAPL", 0)
    assert fixture_server.requests == []


@pytest.mark.parametrize("value", ["yes", 1, 0])
def test_non_bool_active_is_a_type_error(client: Any, fixture_server: FixtureServer, value: Any) -> None:
    """``active`` accepts only ``bool``; other shapes fail before conversion."""
    with pytest.raises(TypeError):
        client.congressional.profiles(active=value)
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("method", "args", "kwargs", "message"),
    [
        pytest.param(
            "senate_trades", ("   ",), {}, "symbol: value must not be empty or whitespace-only", id="ticker"
        ),
        pytest.param("house_trades", ("AAPL,MSFT",), {}, "symbol: ", id="ticker-comma"),
        pytest.param("house_trades_by_name", (" ",), {}, "name: ", id="search-term"),
        pytest.param("net_worth", ("",), {}, "member_id: ", id="member-id-required"),
        pytest.param("senate_trades_by_member_id", (), {"member_id": "  "}, "member_id: ", id="member-id-optional"),
        pytest.param(
            "latest_senate_disclosures", (), {"page": -1}, "page: must be an integer from 0 through 4294967295", id="page"
        ),
        pytest.param(
            "latest_house_disclosures",
            (),
            {"limit": 4_294_967_296},
            "limit: must be an integer from 0 through 4294967295",
            id="limit",
        ),
        pytest.param("profiles", (), {"latest_party": ""}, "latest_party: ", id="text-profiles"),
        pytest.param("positions", (), {"position": "   "}, "position: ", id="text-positions"),
        pytest.param("net_worth_aggregated", (MEMBER_ID,), {"totals_col": ""}, "totals_col: ", id="text-totals-col"),
    ],
)
def test_invalid_values_name_the_argument(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    args: tuple[Any, ...],
    kwargs: dict[str, Any],
    message: str,
) -> None:
    """Each kind family fails locally with ``FmpValidationError`` prefixed by the keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.congressional, method)(*args, **kwargs)
    error = raised.value
    assert str(error).startswith(message)
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_status_failure_is_structured(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-2xx body raises ``FmpStatusError`` carrying the status and the endpoint."""
    fixture_server.route("/senate-profile", {"Error Message": "Premium"}, status=402)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.congressional.profiles()
    assert raised.value.status == 402
    assert raised.value.endpoint == "senate-profile"


def test_decode_failure_is_structured(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-JSON 200 body maps to ``FmpDecodeError`` and names the endpoint."""
    fixture_server.route("/senate-net-worth-aggregated", b"not-json", content_type="text/plain")
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.congressional.net_worth_aggregated(MEMBER_ID)
    assert raised.value.endpoint == "senate-net-worth-aggregated"
    assert len(fixture_server.requests) == 1
