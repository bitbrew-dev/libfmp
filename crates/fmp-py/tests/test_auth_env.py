"""``FMP_API_KEY`` pickup when ``FmpClient`` is built without ``token``.

Precedence under test: an explicit ``token`` always wins; otherwise the
variable is used as the credential (``auth_mode`` defaults to ``fmp_header``
and the other modes combine with it); ``auth_mode="none"`` ignores it; with
neither, a custom ``base_url`` selects no auth while the default host raises
``FmpConfigError`` naming the variable.
"""

from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp import FmpClient

QUOTE_SHORT_PATH = "/quote-short"
MISSING_TOKEN_MESSAGE = "no token given and FMP_API_KEY is not set"


def _short_quote(fixture_server: FixtureServer, **configuration: Any) -> Any:
    fixture_server.route(QUOTE_SHORT_PATH, load_fixture("quote_short.json"))
    client = FmpClient(base_url=fixture_server.base_url, path_prefix="", **configuration)
    assert len(client.quote.short("AAPL")) == 1
    return fixture_server.requests[0]


def test_env_key_becomes_the_fmp_header_when_token_is_omitted(
    fixture_server: FixtureServer, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Without ``token`` or ``auth_mode`` the variable is sent as the ``apikey`` header."""
    monkeypatch.setenv("FMP_API_KEY", "shell-secret")
    request = _short_quote(fixture_server)
    assert request.headers["apikey"] == "shell-secret"
    assert "apikey" not in request.query


def test_explicit_token_beats_the_env_key(fixture_server: FixtureServer, monkeypatch: pytest.MonkeyPatch) -> None:
    """A ``token`` argument is used verbatim even when the variable is set."""
    monkeypatch.setenv("FMP_API_KEY", "shell-secret")
    request = _short_quote(fixture_server, token="explicit-secret")
    assert request.headers["apikey"] == "explicit-secret"


ENV_MODES = [
    pytest.param({"auth_mode": "fmp_header"}, ("apikey", "shell-secret"), None, id="header"),
    pytest.param({"auth_mode": "fmp_query"}, None, ("apikey", "shell-secret"), id="query"),
    pytest.param({"auth_mode": "bearer"}, ("authorization", "Bearer shell-secret"), None, id="bearer"),
    pytest.param(
        {"auth_mode": "custom_header", "auth_name": "X-Router-Token", "auth_prefix": "Token "},
        ("x-router-token", "Token shell-secret"),
        None,
        id="custom-header",
    ),
    pytest.param(
        {"auth_mode": "custom_query", "auth_name": "router_token"},
        None,
        ("router_token", "shell-secret"),
        id="custom-query",
    ),
]


@pytest.mark.parametrize(("configuration", "expected_header", "expected_query"), ENV_MODES)
def test_env_key_combines_with_every_token_taking_mode(
    fixture_server: FixtureServer,
    monkeypatch: pytest.MonkeyPatch,
    configuration: dict[str, str],
    expected_header: tuple[str, str] | None,
    expected_query: tuple[str, str] | None,
) -> None:
    """Each mode that accepts ``token`` accepts the variable in its place."""
    monkeypatch.setenv("FMP_API_KEY", "shell-secret")
    request = _short_quote(fixture_server, **configuration)
    if expected_header is not None:
        name, value = expected_header
        assert request.headers[name] == value
    if expected_query is not None:
        name, value = expected_query
        assert request.query[name] == [value]


def test_auth_mode_none_ignores_the_env_key(fixture_server: FixtureServer, monkeypatch: pytest.MonkeyPatch) -> None:
    """``auth_mode="none"`` sends no credential even when the variable is set."""
    monkeypatch.setenv("FMP_API_KEY", "shell-secret")
    request = _short_quote(fixture_server, auth_mode="none")
    assert "apikey" not in request.headers
    assert "authorization" not in request.headers
    assert "apikey" not in request.query


def test_custom_base_url_without_any_key_selects_no_auth(fixture_server: FixtureServer) -> None:
    """Neither ``token`` nor the variable with a custom ``base_url`` keeps today's no-auth path."""
    request = _short_quote(fixture_server)
    assert "apikey" not in request.headers
    assert "apikey" not in request.query


@pytest.mark.parametrize("value", [None, "", "   ", "\t\n"], ids=["unset", "empty", "spaces", "control"])
def test_default_host_without_any_key_names_the_variable(
    errors: SimpleNamespace, monkeypatch: pytest.MonkeyPatch, value: str | None
) -> None:
    """The default host with no credential raises ``FmpConfigError`` naming ``FMP_API_KEY``."""
    if value is not None:
        monkeypatch.setenv("FMP_API_KEY", value)
    with pytest.raises(errors.FmpConfigError) as raised:
        FmpClient()
    error = raised.value
    assert str(error) == MISSING_TOKEN_MESSAGE
    assert error.category == "configuration"
    assert (error.endpoint, error.status, error.body) == (None, None, None)


def test_explicit_default_host_without_any_key_names_the_variable(errors: SimpleNamespace) -> None:
    """Spelling out the default origin as ``base_url`` is still the default host."""
    with pytest.raises(errors.FmpConfigError, match="FMP_API_KEY"):
        FmpClient(base_url="https://financialmodelingprep.com")


def test_default_host_with_env_key_constructs(monkeypatch: pytest.MonkeyPatch) -> None:
    """``FmpClient()`` builds against the default host once the variable is set."""
    monkeypatch.setenv("FMP_API_KEY", "shell-secret")
    FmpClient()


def test_surrounding_whitespace_in_the_env_key_is_trimmed(
    fixture_server: FixtureServer, monkeypatch: pytest.MonkeyPatch
) -> None:
    """A padded value is sent without its surrounding whitespace."""
    monkeypatch.setenv("FMP_API_KEY", "  shell-secret\n")
    request = _short_quote(fixture_server)
    assert request.headers["apikey"] == "shell-secret"


def test_auth_mode_none_on_the_default_host_keeps_the_explicit_auth_error(
    errors: SimpleNamespace, monkeypatch: pytest.MonkeyPatch
) -> None:
    """Explicit ``auth_mode="none"`` wins over the variable and keeps the original message."""
    monkeypatch.setenv("FMP_API_KEY", "shell-secret")
    with pytest.raises(errors.FmpConfigError) as raised:
        FmpClient(auth_mode="none")
    assert str(raised.value) == "direct FMP access requires explicit authentication"


def test_token_taking_mode_without_any_key_keeps_the_token_error(errors: SimpleNamespace) -> None:
    """An explicit token-taking mode with nothing to use keeps its existing message."""
    with pytest.raises(errors.FmpConfigError) as raised:
        FmpClient(auth_mode="bearer", base_url="https://proxy.example")
    assert str(raised.value) == "the selected authentication mode requires token"
