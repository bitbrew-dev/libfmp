"""Runtime contract of ``client.fundraising``: the negative cases and the error mapping.

Every argument kind the domain uses (``cik``, ``search_term``, ``page``,
``limit``) fails locally with ``FmpValidationError`` prefixed by the keyword,
shape errors surface as ``TypeError``, and a non-success status or a non-JSON
body maps to the structured error types. The happy paths live in
``test_fundraising_a.py``.
"""

from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer

NJOY_CIK = "0001547416"
U32_MAX = 4_294_967_295


@pytest.mark.parametrize(
    ("method", "arguments", "keyword"),
    [
        pytest.param("crowdfunding_offerings_by_cik", ("",), "cik", id="crowdfunding-cik-empty"),
        pytest.param("regulation_d_offerings_by_cik", ("  ",), "cik", id="regulation-d-cik-blank"),
        pytest.param("search_crowdfunding_offerings", (" \t ",), "name", id="crowdfunding-name-blank"),
        pytest.param("search_regulation_d_offerings", ("",), "name", id="regulation-d-name-empty"),
    ],
)
def test_blank_identifier_names_the_argument(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    arguments: tuple[Any, ...],
    keyword: str,
) -> None:
    """A blank required CIK or search term is rejected locally with the keyword as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.fundraising, method)(*arguments)
    error = raised.value
    assert str(error) == f"{keyword}: value must not be empty or whitespace-only"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_control_character_in_search_term_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A search term containing a control character is rejected under ``name``."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.fundraising.search_regulation_d_offerings("NJOY\nINC")
    assert str(raised.value).startswith("name: ")
    assert fixture_server.requests == []


def test_blank_optional_cik_names_the_argument(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """The optional ``cik`` setter validates its value under its own keyword."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.fundraising.latest_regulation_d_offerings(cik=" ")
    assert str(raised.value) == "cik: value must not be empty or whitespace-only"
    assert fixture_server.requests == []


@pytest.mark.parametrize(
    ("method", "keyword", "value"),
    [
        pytest.param("latest_crowdfunding_offerings", "page", -1, id="crowdfunding-page-negative"),
        pytest.param("latest_crowdfunding_offerings", "limit", U32_MAX + 1, id="crowdfunding-limit-overflow"),
        pytest.param("latest_regulation_d_offerings", "page", U32_MAX + 1, id="regulation-d-page-overflow"),
        pytest.param("latest_regulation_d_offerings", "limit", -1, id="regulation-d-limit-negative"),
    ],
)
def test_out_of_range_pagination_names_the_argument(
    client: Any,
    fixture_server: FixtureServer,
    errors: SimpleNamespace,
    method: str,
    keyword: str,
    value: int,
) -> None:
    """A ``page`` or ``limit`` outside the u32 domain fails locally with the keyword as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        getattr(client.fundraising, method)(**{keyword: value})
    error = raised.value
    assert str(error) == f"{keyword}: must be an integer from 0 through {U32_MAX}"
    assert error.category == "validation"
    assert fixture_server.requests == []


def test_wrong_argument_types_raise_type_error(client: Any, fixture_server: FixtureServer) -> None:
    """An ``int`` CIK, a ``str`` page, or a ``float`` limit is a shape error reported as ``TypeError``."""
    with pytest.raises(TypeError):
        client.fundraising.regulation_d_offerings_by_cik(1547416)
    with pytest.raises(TypeError):
        client.fundraising.search_crowdfunding_offerings(None)
    with pytest.raises(TypeError):
        client.fundraising.latest_crowdfunding_offerings(page="0")
    with pytest.raises(TypeError):
        client.fundraising.latest_regulation_d_offerings(limit=10.0)
    assert fixture_server.requests == []


def test_status_error_carries_the_endpoint_id(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on ``latest_regulation_d_offerings`` names the endpoint id."""
    fixture_server.route("/fundraising-latest", {"error": "denied"}, status=403)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.fundraising.latest_regulation_d_offerings(cik=NJOY_CIK)
    error = raised.value
    assert error.endpoint == "fundraising-latest"
    assert error.status == 403
    assert error.body == '{"error": "denied"}'


def test_status_error_on_the_bare_fundraising_path(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A non-success status on ``regulation_d_offerings_by_cik`` names the bare ``fundraising`` id."""
    fixture_server.route("/fundraising", {"error": "denied"}, status=402)
    with pytest.raises(errors.FmpStatusError) as raised:
        client.fundraising.regulation_d_offerings_by_cik(NJOY_CIK)
    assert raised.value.endpoint == "fundraising"
    assert raised.value.status == 402


def test_empty_arrays_decode_to_empty_lists(client: Any, fixture_server: FixtureServer) -> None:
    """An empty JSON array on each feed decodes to an empty Python list."""
    fixture_server.route("/crowdfunding-offerings-search", [])
    fixture_server.route("/fundraising-search", [])
    assert client.fundraising.search_crowdfunding_offerings("NJOY") == []
    assert client.fundraising.search_regulation_d_offerings("NJOY") == []
    assert [request.path for request in fixture_server.requests] == [
        "/crowdfunding-offerings-search",
        "/fundraising-search",
    ]


def test_malformed_root_is_a_decode_error(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A JSON object where an array is documented maps to ``FmpDecodeError`` and names the endpoint."""
    fixture_server.route("/crowdfunding-offerings", {})
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.fundraising.crowdfunding_offerings_by_cik("0001916078")
    assert raised.value.endpoint == "crowdfunding-offerings"


def test_non_json_body_is_a_decode_error(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A non-JSON 200 body maps to ``FmpDecodeError`` and names the endpoint."""
    fixture_server.route("/crowdfunding-offerings-latest", b"not-json", content_type="text/plain")
    with pytest.raises(errors.FmpDecodeError) as raised:
        client.fundraising.latest_crowdfunding_offerings()
    assert raised.value.endpoint == "crowdfunding-offerings-latest"
