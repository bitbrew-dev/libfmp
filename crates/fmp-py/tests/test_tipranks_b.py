"""Runtime contract of ``client.tipranks`` for the negatives and the error mapping.

The negative cases cover every argument kind the domain has (``ticker``,
``tipranks_expert_uid``, ``search_term``, ``date``, ``page``, ``limit``, and
the bare ``boolean`` flag, which is a ``TypeError`` rather than a validation
error), the keyword-only call shapes, and the structured status and decode
failures. The endpoint ids in the errors are the ones the Rust
``tipranks_*_endpoints.rs`` tests pin.
"""

from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture

U32_MAX = 4_294_967_295
BLANK_MESSAGE = "value must not be empty or whitespace-only"


@pytest.mark.parametrize(
    ("method", "args", "kwargs", "message"),
    [
        pytest.param("point_in_time_ratings_by_symbol", ("  ",), {}, f"symbol: {BLANK_MESSAGE}", id="ticker-required"),
        pytest.param("search_ratings", (), {"symbol": ""}, f"symbol: {BLANK_MESSAGE}", id="ticker-optional"),
        pytest.param("analyst_summary", ("  ",), {}, f"expert_uid: {BLANK_MESSAGE}", id="expert-uid-required"),
        pytest.param(
            "search_ratings", (), {"expert_uid": ""}, f"expert_uid: {BLANK_MESSAGE}", id="expert-uid-optional"
        ),
        pytest.param("firm_summary", ("",), {}, f"firm_name: {BLANK_MESSAGE}", id="search-term-required"),
        pytest.param(
            "analysts", (), {"analyst_name": "  "}, f"analyst_name: {BLANK_MESSAGE}", id="search-term-optional"
        ),
        pytest.param(
            "point_in_time_ratings_by_analyst",
            (),
            {"analyst_name": "Keegan\nCox"},
            "analyst_name: value must not contain control characters",
            id="search-term-control-character",
        ),
        pytest.param(
            "symbol_summary",
            ("AAPL",),
            {"from_": "06/10/2025"},
            "from_: value must be a valid YYYY-MM-DD date",
            id="from",
        ),
        pytest.param(
            "firm_summary",
            ("Morgan Stanley",),
            {"to": "2026-13-01"},
            "to: value must be a valid YYYY-MM-DD date",
            id="to",
        ),
        pytest.param(
            "point_in_time_ratings_by_symbol",
            ("AAPL",),
            {"date": "2026-6-6"},
            "date: value must be a valid YYYY-MM-DD date",
            id="date",
        ),
        pytest.param(
            "search_ratings", (), {"page": -1}, f"page: must be an integer from 0 through {U32_MAX}", id="page"
        ),
        pytest.param(
            "analysts", (), {"limit": U32_MAX + 1}, f"limit: must be an integer from 0 through {U32_MAX}", id="limit"
        ),
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
    """Each kind family fails locally with ``FmpValidationError`` prefixed by the argument name."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.tipranks, method)(*args, **kwargs)
    error = raised.value
    assert str(error) == message
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize("value", ["false", 0, [True]], ids=["string", "int", "list"])
def test_bool_typed_flag_rejects_non_bool_shapes(client: Any, fixture_server: FixtureServer, value: Any) -> None:
    """``nonadjusted`` (``boolean``) accepts only ``bool`` on all three methods that expose it."""
    with pytest.raises(TypeError):
        client.tipranks.search_ratings(nonadjusted=value)
    with pytest.raises(TypeError):
        client.tipranks.point_in_time_ratings_by_symbol("AAPL", nonadjusted=value)
    with pytest.raises(TypeError):
        client.tipranks.point_in_time_ratings_by_analyst(nonadjusted=value)
    assert fixture_server.requests == []


def test_summary_methods_reject_the_flag(client: Any, fixture_server: FixtureServer) -> None:
    """The summary queries have no ``nonadjusted`` keyword, so passing one is a ``TypeError``."""
    with pytest.raises(TypeError):
        client.tipranks.symbol_summary("AAPL", nonadjusted=True)
    with pytest.raises(TypeError):
        client.tipranks.analysts(nonadjusted=False)
    assert fixture_server.requests == []


def test_optionals_are_keyword_only(client: Any, fixture_server: FixtureServer) -> None:
    """A positional value where the query has only setters is a ``TypeError``, never a silent filter."""
    with pytest.raises(TypeError):
        client.tipranks.search_ratings("RR.L")
    with pytest.raises(TypeError):
        client.tipranks.symbol_summary("AAPL", "2025-06-10")
    with pytest.raises(TypeError):
        client.tipranks.analysts(0)
    assert fixture_server.requests == []


def test_from_keyword_is_renamed_but_the_wire_key_is_not(client: Any, fixture_server: FixtureServer) -> None:
    """The Python keyword is ``from_``; ``from`` is rejected and the wire key stays ``from``."""
    fixture_server.route("/tipranks-symbol-summary", load_fixture("tipranks_symbol_summary.json"))
    with pytest.raises(TypeError, match="unexpected keyword argument 'from'"):
        client.tipranks.symbol_summary("AAPL", **{"from": "2025-06-10"})
    assert fixture_server.requests == []

    client.tipranks.symbol_summary("AAPL", from_="2025-06-10")
    assert fixture_server.requests[0].query == {"symbol": ["AAPL"], "from": ["2025-06-10"]}
    assert "from_" not in fixture_server.requests[0].raw_query


def test_required_arguments_cannot_be_omitted(client: Any, fixture_server: FixtureServer) -> None:
    """The constructor arguments (``symbol``, ``expert_uid``, ``firm_name``) are mandatory."""
    with pytest.raises(TypeError):
        client.tipranks.point_in_time_ratings_by_symbol()
    with pytest.raises(TypeError):
        client.tipranks.analyst_summary(from_="2025-06-10")
    with pytest.raises(TypeError):
        client.tipranks.firm_summary()
    assert fixture_server.requests == []


def test_status_error_names_the_search_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status carries the libfmp endpoint id, status, and body."""
    fixture_server.route("/tipranks-search", {"error": "add-on required"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.tipranks.search_ratings(symbol="RR.L")
    error = raised.value
    assert error.endpoint == "tipranks-search"
    assert error.status == 403
    assert error.body == '{"error": "add-on required"}'


def test_status_error_names_the_directory_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-JSON 500 on the directory route still maps to a structured status error."""
    fixture_server.route("/tipranks-analysts", b"not-json", status=500, content_type="text/plain")
    with pytest.raises(errors.FmpStatusError) as raised:
        client.tipranks.analysts()
    assert raised.value.endpoint == "tipranks-analysts"
    assert raised.value.status == 500
    assert raised.value.body == "not-json"


@pytest.mark.parametrize(
    ("method", "args", "path", "endpoint"),
    [
        pytest.param(
            "point_in_time_ratings_by_analyst", (), "/tipranks-pit-analyst", "tipranks-pit-analyst", id="pit-analyst"
        ),
        pytest.param(
            "point_in_time_ratings_by_symbol", ("AAPL",), "/tipranks-pit-symbol", "tipranks-pit-symbol", id="pit-symbol"
        ),
        pytest.param(
            "analyst_summary",
            ("expert",),
            "/tipranks-analyst-summary",
            "tipranks-analyst-summary",
            id="analyst-summary",
        ),
    ],
)
def test_malformed_root_is_a_decode_error_naming_the_endpoint(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    args: tuple[Any, ...],
    path: str,
    endpoint: str,
) -> None:
    """A ``{}`` root where an array is documented surfaces as a decode error with the endpoint id."""
    fixture_server.route(path, {})
    with pytest.raises(errors.FmpDecodeError) as raised:
        getattr(client.tipranks, method)(*args)
    assert raised.value.endpoint == endpoint


def test_wrongly_shaped_row_is_a_decode_error(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A well-formed array whose row misses the nested counts is a decode error on the firm route."""
    fixture_server.route("/tipranks-firm-summary", [{"firmName": "Morgan Stanley", "from": "2025-07-30"}])
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.tipranks.firm_summary("Morgan Stanley")
    assert raised.value.endpoint == "tipranks-firm-summary"


def test_empty_array_decodes_to_no_rows(client: Any, fixture_server: FixtureServer) -> None:
    """A bare ``[]`` body decodes to an empty list on every route (the default route body)."""
    assert client.tipranks.search_ratings() == []
    assert client.tipranks.symbol_summary("AAPL") == []
    assert client.tipranks.analysts() == []
    assert [request.path for request in fixture_server.requests] == [
        "/tipranks-search",
        "/tipranks-symbol-summary",
        "/tipranks-analysts",
    ]
