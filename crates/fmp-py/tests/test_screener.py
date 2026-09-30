"""Runtime contract of ``client.screener``, the filter-shape pilot.

``companies`` takes twenty keyword-only optional filters. One test sends none
(bare path), one sets every filter and asserts the documented encoding order,
one proves ``0`` and ``False`` are emitted rather than dropped, and the
negatives cover one invalid value per argument-kind family. Expected targets
come from the Rust ``screener_endpoint.rs`` test, with one deviation: the
Rust test pins ``u64::MAX`` for the two ``*_lower_than`` integer filters,
which the ``int`` -> ``i64`` boundary cannot carry, so this file pins
``2**63 - 1`` there and asserts the overflow separately.
"""

from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.screener import CompanyScreenerResult, ScreenerNamespace

I64_MAX = 2**63 - 1

ALL_FILTERS_TARGET = (
    "/company-screener"
    "?marketCapMoreThan=9007199254740993"
    f"&marketCapLowerThan={I64_MAX}"
    "&sector=Technology+%26+AI"
    "&industry=Consumer+Electronics+%2F+Devices"
    "&betaMoreThan=0.5&betaLowerThan=1.5"
    "&priceMoreThan=10.25&priceLowerThan=500"
    "&dividendMoreThan=0.5&dividendLowerThan=2"
    "&volumeMoreThan=1000"
    f"&volumeLowerThan={I64_MAX}"
    "&exchange=NASDAQ+Global"
    "&country=US+%2F+CA"
    "&isEtf=false&isFund=false&isActivelyTrading=true"
    "&page=0&limit=4294967295&includeAllShareClasses=false"
)

ZERO_AND_FALSE_TARGET = (
    "/company-screener?marketCapMoreThan=0&priceMoreThan=0&volumeMoreThan=0"
    "&isEtf=false&isFund=false&isActivelyTrading=false&page=0&limit=0&includeAllShareClasses=false"
)


def test_screener_namespace_is_the_generated_type(client: Any) -> None:
    """``client.screener`` is the generated namespace class."""
    assert isinstance(client.screener, ScreenerNamespace)


def test_companies_without_filters_sends_the_bare_path(client: Any, fixture_server: FixtureServer) -> None:
    """No filters means no query string at all, and the documented body decodes."""
    fixture_server.route("/company-screener", load_fixture("company_screener.json"))
    rows = client.screener.companies()

    assert fixture_server.requests[0].target == "/company-screener"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, CompanyScreenerResult)
    assert row.symbol == "AAPL"
    assert row.company_name == "Apple Inc."
    assert row.market_cap == 4_885_602_246_714
    assert row.sector == "Technology"
    assert row.industry == "Consumer Electronics"
    assert row.beta == pytest.approx(1.097)
    assert row.price == pytest.approx(332.64001)
    assert row.last_annual_dividend == pytest.approx(1.05)
    assert row.volume == 29_909_012
    assert row.exchange == "NASDAQ Global Select"
    assert row.exchange_short_name == "NASDAQ"
    assert row.country == "US"
    assert row.is_etf is False
    assert row.is_fund is False
    assert row.is_actively_trading is True


def test_companies_with_every_filter_encodes_in_documented_order(client: Any, fixture_server: FixtureServer) -> None:
    """All twenty setters are reachable as keywords and encode in the Rust order."""
    fixture_server.route("/company-screener", load_fixture("company_screener.json"))
    rows = client.screener.companies(
        market_cap_more_than=9_007_199_254_740_993,
        market_cap_lower_than=I64_MAX,
        sector="Technology & AI",
        industry="Consumer Electronics / Devices",
        beta_more_than=0.5,
        beta_lower_than=1.5,
        price_more_than=10.25,
        price_lower_than=500.0,
        dividend_more_than=0.5,
        dividend_lower_than=2.0,
        volume_more_than=1_000,
        volume_lower_than=I64_MAX,
        exchange="NASDAQ Global",
        country="US / CA",
        is_etf=False,
        is_fund=False,
        is_actively_trading=True,
        page=0,
        limit=4_294_967_295,
        include_all_share_classes=False,
    )

    assert fixture_server.requests[0].target == ALL_FILTERS_TARGET
    assert len(rows) == 1
    assert isinstance(rows[0], CompanyScreenerResult)
    assert rows[0].symbol == "AAPL"


def test_companies_emits_zero_and_false_but_omits_absent_filters(client: Any, fixture_server: FixtureServer) -> None:
    """``0`` and ``False`` are real filter values; unset keywords never appear."""
    fixture_server.route("/company-screener", load_fixture("company_screener_empty.json"))
    rows = client.screener.companies(
        market_cap_more_than=0,
        price_more_than=0.0,
        volume_more_than=0,
        is_etf=False,
        is_fund=False,
        is_actively_trading=False,
        page=0,
        limit=0,
        include_all_share_classes=False,
    )

    assert fixture_server.requests[0].target == ZERO_AND_FALSE_TARGET
    assert rows == []


def test_companies_preserves_large_integers_and_flags(client: Any, fixture_server: FixtureServer) -> None:
    """``market_cap`` and ``volume`` are floats: ``2**53 + 1`` rounds to ``2**53`` and ``u64::MAX`` to ``2**64``."""
    fixture_server.route("/company-screener", load_fixture("company_screener_multiple.json"))
    rows = client.screener.companies(limit=2)

    assert fixture_server.requests[0].target == "/company-screener?limit=2"
    assert len(rows) == 2
    assert rows[0].symbol == "BIG"
    assert isinstance(rows[0].market_cap, float)
    assert rows[0].market_cap == float(9_007_199_254_740_993)
    assert isinstance(rows[0].volume, float)
    assert rows[0].volume == float(18_446_744_073_709_551_615)
    assert rows[0].sector == "Future Sector"
    assert rows[0].is_actively_trading is False
    assert rows[1].symbol == "FUND"
    assert rows[1].market_cap == 4_294_967_296
    assert rows[1].is_fund is True
    assert rows[1].last_annual_dividend == pytest.approx(2.5)


def test_companies_rejects_positional_arguments(client: Any, fixture_server: FixtureServer) -> None:
    """Every filter is keyword-only; a positional value is a ``TypeError``."""
    with pytest.raises(TypeError):
        client.screener.companies(1_000_000)
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("keyword", "value", "message"),
    [
        ("market_cap_more_than", -1, "market_cap_more_than: must be a non-negative integer"),
        ("volume_lower_than", -5, "volume_lower_than: must be a non-negative integer"),
        ("limit", -1, "limit: must be an integer from 0 through 4294967295"),
        ("page", 2**32, "page: must be an integer from 0 through 4294967295"),
        ("beta_more_than", float("nan"), "beta_more_than: decimal value must be finite"),
        ("price_lower_than", float("inf"), "price_lower_than: decimal value must be finite"),
        ("sector", "", "sector: value must not be empty or whitespace-only"),
        ("industry", "   ", "industry: value must not be empty or whitespace-only"),
        ("exchange", "", "exchange: value must not be empty or whitespace-only"),
        ("country", "\t", "country: value must not be empty or whitespace-only"),
    ],
)
def test_invalid_filter_values_name_the_keyword(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, keyword: str, value: Any, message: str
) -> None:
    """One invalid value per kind family fails locally with the keyword as prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.screener.companies(**{keyword: value})
    error = raised.value
    assert str(error) == message
    assert str(error).startswith(f"{keyword}: ")
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize("value", ["yes", 1, 0])
def test_non_bool_flag_is_a_type_error(client: Any, fixture_server: FixtureServer, value: Any) -> None:
    """The boolean filters accept only ``bool``; other shapes fail before conversion."""
    with pytest.raises(TypeError):
        client.screener.companies(is_etf=value)
    assert fixture_server.requests == []


@pytest.mark.parametrize("keyword", ["market_cap_lower_than", "volume_lower_than"])
def test_integer_filters_above_i64_overflow_before_validation(
    client: Any, fixture_server: FixtureServer, keyword: str
) -> None:
    """``u64::MAX`` is unreachable from Python: the ``i64`` boundary raises ``OverflowError``."""
    with pytest.raises(OverflowError):
        client.screener.companies(**{keyword: 2**64 - 1})
    assert fixture_server.requests == []


def test_status_error_carries_the_screener_endpoint_id(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status maps to ``FmpStatusError`` with the endpoint id."""
    fixture_server.route("/company-screener", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.screener.companies(sector="Technology")
    error = raised.value
    assert error.endpoint == "company-screener"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_decode_error_for_a_non_json_body(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A body that is not JSON maps to ``FmpDecodeError`` rather than a raw exception."""
    fixture_server.route("/company-screener", b"not-json " * 40)
    with pytest.raises(errors.FmpDecodeError) as caught:
        client.screener.companies()

    assert (caught.value.decode_kind, caught.value.decode_path) == ("syntax", None)


def test_companies_decodes_a_page_with_a_null_beta_row(client: Any, fixture_server: FixtureServer) -> None:
    """Issue #368: a ``null`` beta no longer fails the page; the row's nullable members read ``None``."""
    rows = load_fixture("company_screener_null_beta_synthetic.json")
    for member in ("price", "lastAnnualDividend", "country", "isFund"):
        rows[36][member] = None
    fixture_server.route("/company-screener", rows)
    decoded = client.screener.companies()

    assert len(decoded) == 40
    assert decoded[37].beta is None
    assert decoded[35].beta == pytest.approx(1.097)
    assert (decoded[36].price, decoded[36].last_annual_dividend, decoded[36].country, decoded[36].is_fund) == (
        None,
        None,
        None,
        None,
    )


def test_decode_error_names_the_null_member_row_and_member(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """Issue #366: a ``null`` required member names its row index and member."""
    rows = load_fixture("company_screener_null_beta_synthetic.json")
    rows[37]["companyName"] = None
    fixture_server.route("/company-screener", rows)
    with pytest.raises(errors.FmpDecodeError) as caught:
        client.screener.companies()

    assert caught.value.decode_path == "[37].companyName"
    assert caught.value.decode_kind == "null"
    assert str(caught.value).startswith(
        "successful response could not be decoded: null value at [37].companyName (endpoint: company-screener): "
    )


def test_decode_error_never_carries_a_string_member_value(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """Issue #366: the offending string value reaches neither the message nor an attribute."""
    sentinel = "SENTINEL-beta-text"
    rows = load_fixture("company_screener_null_beta_synthetic.json")
    rows[37]["volume"] = sentinel
    fixture_server.route("/company-screener", rows)
    with pytest.raises(errors.FmpDecodeError) as caught:
        client.screener.companies()

    error = caught.value
    assert error.decode_path == "[37].volume"
    assert error.decode_kind == "wrong_type"
    assert error.body_truncated is True
    assert sentinel not in str(error)
    assert sentinel not in repr(error)
    assert all(sentinel not in str(value) for value in vars(error).values())


def test_companies_decodes_integral_float_and_fractional_market_caps(client: Any, fixture_server: FixtureServer) -> None:
    """Issue #339: an integral-float or fractional ``marketCap`` decodes as a ``float``."""
    fixture_server.route("/company-screener", load_fixture("company_screener_fractional_synthetic.json"))
    rows = client.screener.companies()

    assert [row.symbol for row in rows] == ["AAPL", "FRAC"]
    assert isinstance(rows[0].market_cap, float)
    assert rows[0].market_cap == 4_885_602_246_714.0
    assert rows[1].market_cap == 1_234_567.5
    assert rows[1].volume == 1.0
