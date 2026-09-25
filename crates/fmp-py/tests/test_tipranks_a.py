"""Runtime contract of ``client.tipranks`` for the seven add-on methods.

One test per method routes the documented fixture body, calls the method with
one argument shape, and asserts the exact request target plus a few typed
fields (including the ``datetime.date`` ones and the nested count rows). The
expected targets are the ones the Rust ``tipranks_*_endpoints.rs`` tests pin.
The ``date`` field on the search and point-in-time rows is the provider's ISO
timestamp preserved as ``str``; the JSON ``Number`` fields (price targets,
returns, rates) reach Python as ``int`` or ``float`` depending on the wire
value. The negative cases live in ``test_tipranks_b.py``.
"""

import datetime
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.tipranks import (
    TipRanksAnalystActionCounts,
    TipRanksAnalystProfile,
    TipRanksAnalystSummary,
    TipRanksFirmSummary,
    TipranksNamespace,
    TipRanksPointInTimeRating,
    TipRanksRatingSearchResult,
    TipRanksRecommendationCounts,
    TipRanksSymbolSummary,
)

EXPERT_UID = "expert / one"
ENCODED_EXPERT_UID = "expertUID=expert+%2F+one"
SUMMARY_FROM = datetime.date(2025, 6, 10)
SUMMARY_TO = datetime.date(2026, 6, 10)
SUMMARY_RANGE = "from=2025-06-10&to=2026-06-10"


def test_tipranks_namespace_is_the_generated_type(client: Any) -> None:
    """``client.tipranks`` is the generated flat namespace class."""
    assert isinstance(client.tipranks, TipranksNamespace)


def test_ratings_search_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``search_ratings`` encodes every optional in wire order and decodes the rating row."""
    fixture_server.route("/tipranks-search", load_fixture("tipranks_ratings_search.json"))
    rows = client.tipranks.search_ratings(
        expert_uid=EXPERT_UID,
        symbol="RR.L",
        from_=SUMMARY_FROM,
        to="2026-06-10",
        limit=5000,
        page=0,
        nonadjusted=False,
    )

    assert fixture_server.requests[0].target == (
        f"/tipranks-search?{ENCODED_EXPERT_UID}&symbol=RR.L&{SUMMARY_RANGE}&limit=5000&page=0&nonadjusted=false"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, TipRanksRatingSearchResult)
    assert row.symbol == "RR.L"
    assert row.date == "2026-07-30T16:40:58.403Z"
    assert row.recommendation_date == datetime.date(2026, 7, 30)
    assert row.expert_uid == "9d6962cbd29862b8d70de0a2ddb3eb0bdfedc2b7"
    assert row.analyst_name == "Ross Law"
    assert row.firm_name == "Morgan Stanley"
    assert row.recommendation == "buy"
    assert row.analyst_action == "maintained"
    assert isinstance(row.price_target, int)
    assert row.price_target == 1500
    assert row.price_target_currency == "GBX"


def test_ratings_search_takes_no_required_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``search_ratings`` with nothing set hits the bare path with no query string."""
    fixture_server.route("/tipranks-search", load_fixture("tipranks_ratings_search.json"))
    rows = client.tipranks.search_ratings()

    assert fixture_server.requests[0].target == "/tipranks-search"
    assert len(rows) == 1


def test_ratings_search_encodes_a_true_flag_independently(client: Any, fixture_server: FixtureServer) -> None:
    """``nonadjusted=True`` reaches the wire as ``true`` and the other optionals stay out."""
    fixture_server.route("/tipranks-search", load_fixture("tipranks_ratings_search.json"))
    client.tipranks.search_ratings(symbol="RR.L", nonadjusted=True)

    assert fixture_server.requests[0].target == "/tipranks-search?symbol=RR.L&nonadjusted=true"


def test_point_in_time_ratings_by_symbol_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``point_in_time_ratings_by_symbol`` encodes ``symbol``, ``date``, paging, and the flag in that order."""
    fixture_server.route("/tipranks-pit-symbol", load_fixture("tipranks_point_in_time_symbol.json"))
    rows = client.tipranks.point_in_time_ratings_by_symbol(
        "BRK.B", date=SUMMARY_TO, limit=5000, page=0, nonadjusted=False
    )

    assert fixture_server.requests[0].target == (
        "/tipranks-pit-symbol?symbol=BRK.B&date=2026-06-10&limit=5000&page=0&nonadjusted=false"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, TipRanksPointInTimeRating)
    assert row.symbol == "AAPL"
    assert row.date == "2026-07-29T09:30:14.797Z"
    assert row.last_recommendation_date == datetime.date(2026, 7, 28)
    assert row.analyst_name == "Wamsi Mohan"
    assert row.last_recommendation == "buy"
    assert row.last_analyst_action == "reiterated"
    assert isinstance(row.price_target, int)
    assert row.price_target == 380
    assert row.price_target_currency == "USD"
    assert row.stock_success_rate == pytest.approx(0.788)
    assert row.stock_return == pytest.approx(-0.1253)
    assert row.beat_target is False


def test_point_in_time_ratings_by_symbol_with_symbol_only(client: Any, fixture_server: FixtureServer) -> None:
    """``point_in_time_ratings_by_symbol`` sends only ``symbol`` when every optional is left out."""
    fixture_server.route("/tipranks-pit-symbol", load_fixture("tipranks_point_in_time_symbol.json"))
    rows = client.tipranks.point_in_time_ratings_by_symbol("AAPL")

    assert fixture_server.requests[0].target == "/tipranks-pit-symbol?symbol=AAPL"
    assert len(rows) == 1


def test_point_in_time_ratings_by_analyst_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``point_in_time_ratings_by_analyst`` keeps both selectors and decodes the null-heavy row."""
    fixture_server.route("/tipranks-pit-analyst", load_fixture("tipranks_point_in_time_analyst.json"))
    rows = client.tipranks.point_in_time_ratings_by_analyst(
        expert_uid=EXPERT_UID,
        analyst_name="Keegan Cox",
        date="2026-06-10",
        limit=100,
        page=0,
        nonadjusted=False,
    )

    assert fixture_server.requests[0].target == (
        f"/tipranks-pit-analyst?{ENCODED_EXPERT_UID}&analystName=Keegan+Cox&date=2026-06-10"
        "&limit=100&page=0&nonadjusted=false"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, TipRanksPointInTimeRating)
    assert row.symbol == "0J3K.L"
    assert row.expert_uid == "3c6eb8cf1347a4e5757e628fccb684a93abee587"
    assert row.analyst_name == "Keegan Cox"
    assert row.firm_name == "D.A. Davidson"
    assert row.last_recommendation == "Hold"
    assert row.last_recommendation_date == datetime.date(2026, 5, 21)
    assert row.stock_success_rate == 0
    assert row.price_target is None
    assert row.price_target_currency is None
    assert row.stock_return is None
    assert row.beat_target is None


def test_point_in_time_ratings_by_analyst_takes_no_required_arguments(
    client: Any, fixture_server: FixtureServer
) -> None:
    """``point_in_time_ratings_by_analyst`` with nothing set injects no provider defaults."""
    fixture_server.route("/tipranks-pit-analyst", load_fixture("tipranks_point_in_time_analyst.json"))
    rows = client.tipranks.point_in_time_ratings_by_analyst()

    assert fixture_server.requests[0].target == "/tipranks-pit-analyst"
    assert len(rows) == 1


def test_symbol_summary_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``symbol_summary`` encodes ``symbol``, ``from``, ``to`` and decodes the nested count rows."""
    fixture_server.route("/tipranks-symbol-summary", load_fixture("tipranks_symbol_summary.json"))
    rows = client.tipranks.symbol_summary("BRK.B", from_=SUMMARY_FROM, to=SUMMARY_TO)

    assert fixture_server.requests[0].target == f"/tipranks-symbol-summary?symbol=BRK.B&{SUMMARY_RANGE}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, TipRanksSymbolSummary)
    assert row.symbol == "AAPL"
    assert row.from_ == datetime.date(2025, 7, 30)
    assert row.to == datetime.date(2026, 7, 30)
    assert row.total_recommendations == 425
    assert row.distinct_symbols == 1
    assert isinstance(row.recommendations, TipRanksRecommendationCounts)
    assert row.recommendations.buy == 277
    assert isinstance(row.analyst_action, TipRanksAnalystActionCounts)
    assert row.analyst_action.maintained == 309
    assert row.average_return == pytest.approx(0.1697)
    assert row.worst_return == pytest.approx(-0.169)


def test_symbol_summary_with_symbol_only(client: Any, fixture_server: FixtureServer) -> None:
    """``symbol_summary`` sends only ``symbol`` when both dates are left out."""
    fixture_server.route("/tipranks-symbol-summary", load_fixture("tipranks_symbol_summary.json"))
    rows = client.tipranks.symbol_summary("AAPL")

    assert fixture_server.requests[0].target == "/tipranks-symbol-summary?symbol=AAPL"
    assert len(rows) == 1


def test_analyst_summary_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``analyst_summary`` encodes the required ``expertUID`` first and decodes the analyst row."""
    fixture_server.route("/tipranks-analyst-summary", load_fixture("tipranks_analyst_summary.json"))
    rows = client.tipranks.analyst_summary(EXPERT_UID, from_="2025-06-10", to=SUMMARY_TO)

    assert fixture_server.requests[0].target == f"/tipranks-analyst-summary?{ENCODED_EXPERT_UID}&{SUMMARY_RANGE}"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, TipRanksAnalystSummary)
    assert row.expert_uid == "3c6eb8cf1347a4e5757e628fccb684a93abee587"
    assert row.from_ == datetime.date(2025, 7, 30)
    assert row.to == datetime.date(2026, 7, 30)
    assert row.total_recommendations == 22
    assert row.recommendations.hold == 6
    assert row.analyst_action.reiterated == 1
    assert row.top_return == pytest.approx(0.3284)


def test_analyst_summary_omits_to_independently(client: Any, fixture_server: FixtureServer) -> None:
    """``analyst_summary`` accepts a ``from_`` alone and leaves ``to`` out."""
    fixture_server.route("/tipranks-analyst-summary", load_fixture("tipranks_analyst_summary.json"))
    client.tipranks.analyst_summary("expert", from_=SUMMARY_FROM)

    assert fixture_server.requests[0].target == "/tipranks-analyst-summary?expertUID=expert&from=2025-06-10"


def test_firm_summary_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``firm_summary`` encodes the required ``firmName`` first and decodes the firm row."""
    fixture_server.route("/tipranks-firm-summary", load_fixture("tipranks_firm_summary.json"))
    rows = client.tipranks.firm_summary("Morgan Stanley / Asia", from_=SUMMARY_FROM, to=SUMMARY_TO)

    assert fixture_server.requests[0].target == (
        f"/tipranks-firm-summary?firmName=Morgan+Stanley+%2F+Asia&{SUMMARY_RANGE}"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, TipRanksFirmSummary)
    assert row.firm_name == "Morgan Stanley"
    assert row.from_ == datetime.date(2025, 7, 30)
    assert row.to == datetime.date(2026, 7, 30)
    assert row.total_recommendations == 14_182
    assert row.beats == 5_133
    assert row.recommendations.sell == 1_729
    assert row.analyst_action.downgraded == 448
    assert row.average_return == pytest.approx(-0.0299)
    assert isinstance(row.worst_return, int)
    assert row.worst_return == -1


def test_firm_summary_omits_from_independently(client: Any, fixture_server: FixtureServer) -> None:
    """``firm_summary`` accepts a ``to`` alone and leaves ``from`` out."""
    fixture_server.route("/tipranks-firm-summary", load_fixture("tipranks_firm_summary.json"))
    client.tipranks.firm_summary("Morgan Stanley", to="2026-06-10")

    assert fixture_server.requests[0].target == "/tipranks-firm-summary?firmName=Morgan+Stanley&to=2026-06-10"


def test_analysts_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``analysts`` encodes ``page`` before ``limit`` (the provider's order) and decodes the profile row."""
    fixture_server.route("/tipranks-analysts", load_fixture("tipranks_analysts.json"))
    rows = client.tipranks.analysts(
        page=0, limit=1000, firm_name="Morgan Stanley / Asia", analyst_name="Andrew Marok / Exact"
    )

    assert fixture_server.requests[0].target == (
        "/tipranks-analysts?page=0&limit=1000&firmName=Morgan+Stanley+%2F+Asia&analystName=Andrew+Marok+%2F+Exact"
    )
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, TipRanksAnalystProfile)
    assert row.expert_uid == "0458d251af4db6d595c17e02da3bc6ae4bb093b0"
    assert row.analyst_name == "Sujeeva De Silva"
    assert row.firm_name == "Roth MKM"
    assert row.success_rate == pytest.approx(0.617)
    assert row.excess_return == pytest.approx(0.563)
    assert row.total_recommendations == 496
    assert row.good_recommendations == 306
    assert row.analyst_rank == 24
    assert row.num_of_stars == 5


def test_analysts_takes_no_required_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """``analysts`` with nothing set hits the bare directory path."""
    fixture_server.route("/tipranks-analysts", load_fixture("tipranks_analysts.json"))
    rows = client.tipranks.analysts()

    assert fixture_server.requests[0].target == "/tipranks-analysts"
    assert len(rows) == 1
