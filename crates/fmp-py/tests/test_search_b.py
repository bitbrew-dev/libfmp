"""Runtime contract of ``client.search`` for the negatives and the error mapping.

The negative cases cover one invalid value per argument kind the domain uses:
``search_term`` (``query``), ``exchange_code``, ``limit``, ``cik``, ``cusip``,
``isin``, and ``ticker`` (``symbol``). Every string kind rejects an empty or
whitespace-only value locally, before any request, with the argument name as
the message prefix; ``limit`` is bounded to the ``u32`` range. The endpoint
ids in the errors are the ones the Rust ``search_endpoints.rs`` tests pin.
"""

from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer

EMPTY_MESSAGE = "value must not be empty or whitespace-only"


@pytest.mark.parametrize("value", ["", "   "], ids=["empty", "whitespace"])
@pytest.mark.parametrize("method", ["symbol", "name"])
def test_blank_query_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, value: str
) -> None:
    """A blank ``query`` (``search_term``) fails locally and names ``query``."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.search, method)(value)
    error = raised.value
    assert str(error) == f"query: {EMPTY_MESSAGE}"
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["symbol", "name"])
def test_blank_exchange_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str
) -> None:
    """A whitespace-only ``exchange`` (``exchange_code``) fails locally and names ``exchange``."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.search, method)("AAPL", exchange="  ")
    error = raised.value
    assert str(error) == f"exchange: {EMPTY_MESSAGE}"
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("method", "argument", "value"),
    [
        pytest.param("symbol", "AAPL", -1, id="symbol-negative"),
        pytest.param("name", "Apple", 4_294_967_296, id="name-overflow"),
        pytest.param("cik", "320193", -1, id="cik-negative"),
    ],
)
def test_out_of_range_limit_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, argument: str, value: int
) -> None:
    """A ``limit`` outside the ``u32`` range fails locally and names ``limit``."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.search, method)(argument, limit=value)
    error = raised.value
    assert str(error) == "limit: must be an integer from 0 through 4294967295"
    assert error.category == "validation"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("method", "argument"),
    [
        pytest.param("cik", "cik", id="cik"),
        pytest.param("cusip", "cusip", id="cusip"),
        pytest.param("isin", "isin", id="isin"),
        pytest.param("exchange_variants", "symbol", id="ticker"),
    ],
)
def test_blank_identifier_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace, method: str, argument: str
) -> None:
    """A blank required identifier fails locally with its own argument name as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.search, method)("   ")
    error = raised.value
    assert str(error) == f"{argument}: {EMPTY_MESSAGE}"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_optional_filters_are_keyword_only(client: Any, fixture_server: FixtureServer) -> None:
    """``limit`` and ``exchange`` are keyword-only; a second positional value is a ``TypeError``."""
    with pytest.raises(TypeError):
        client.search.symbol("AAPL", 10)
    with pytest.raises(TypeError):
        client.search.name("Apple", 10, "NASDAQ")
    with pytest.raises(TypeError):
        client.search.cik("320193", 50)
    assert fixture_server.requests == []


@pytest.mark.parametrize("method", ["cusip", "isin", "exchange_variants"])
def test_single_argument_methods_reject_extra_keywords(client: Any, fixture_server: FixtureServer, method: str) -> None:
    """The identifier-only methods have no ``limit`` keyword."""
    with pytest.raises(TypeError):
        getattr(client.search, method)("AAPL", limit=10)
    assert fixture_server.requests == []


def test_required_argument_is_mandatory(client: Any, fixture_server: FixtureServer) -> None:
    """Calling any search method without its required value is a ``TypeError``."""
    for method in ("symbol", "name", "cik", "cusip", "isin", "exchange_variants"):
        with pytest.raises(TypeError):
            getattr(client.search, method)()
    assert fixture_server.requests == []


def test_status_error_names_the_symbol_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on ``symbol`` maps to ``FmpStatusError`` naming ``search-symbol``."""
    fixture_server.route("/search-symbol", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.search.symbol("AAPL", limit=1)
    error = raised.value
    assert error.endpoint == "search-symbol"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_status_error_names_the_exchange_variants_endpoint(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-JSON 500 body on ``exchange_variants`` is a status error carrying the raw body."""
    fixture_server.route("/search-exchange-variants", b"not-json", status=500, content_type="text/plain")
    with pytest.raises(errors.FmpStatusError) as raised:
        client.search.exchange_variants("AAPL")
    assert raised.value.endpoint == "search-exchange-variants"
    assert raised.value.status == 500
    assert raised.value.body == "not-json"


def test_decode_error_on_a_non_json_body(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-JSON 200 body on ``cik`` maps to ``FmpDecodeError`` naming ``search-cik``."""
    fixture_server.route("/search-cik", b"not-json", content_type="text/plain")
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.search.cik("320193")
    assert raised.value.endpoint == "search-cik"


def test_decode_error_on_a_wrongly_shaped_row(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A well-formed row missing the documented ``marketCap`` field on ``isin`` is a decode error."""
    fixture_server.route("/search-isin", [{"symbol": "AAPL", "name": "Apple Inc.", "isin": "US0378331005"}])
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.search.isin("US0378331005")
    assert raised.value.endpoint == "search-isin"
