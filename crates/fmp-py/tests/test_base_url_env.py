"""``FMP_BASE_URL`` pickup when ``FmpClient`` is built without ``base_url``.

Precedence under test: an explicit ``base_url`` always wins; otherwise the
variable replaces the default host and keeps its path, so a proxy such as
valet needs only ``FMP_API_KEY`` and ``FMP_BASE_URL``. The variable never
changes authentication: the key rules of ``test_auth_env.py`` and the
plaintext-HTTP guard still apply.
"""

from types import SimpleNamespace

import pytest
from conftest import FixtureServer, load_fixture
from fmp import FmpClient

MISSING_TOKEN_MESSAGE = "no token given and FMP_API_KEY is not set"
INSECURE_MESSAGE = "authenticated plaintext HTTP requires an explicit dangerous opt-in"


def test_env_base_and_env_key_reach_the_proxy_path_with_the_apikey_header(
    fixture_server: FixtureServer, monkeypatch: pytest.MonkeyPatch
) -> None:
    """A valet-shaped base keeps its path before ``stable`` and the key rides in ``apikey``."""
    fixture_server.route("/fmp/stable/quote-short", load_fixture("quote_short.json"))
    monkeypatch.setenv("FMP_API_KEY", "vk_example")
    monkeypatch.setenv("FMP_BASE_URL", f"  {fixture_server.base_url}/fmp\n")
    assert len(FmpClient().quote.short("AAPL")) == 1
    request = fixture_server.requests[0]
    assert request.path == "/fmp/stable/quote-short"
    assert request.headers["apikey"] == "vk_example"
    assert "apikey" not in request.query


def test_explicit_base_url_beats_the_env_base(fixture_server: FixtureServer, monkeypatch: pytest.MonkeyPatch) -> None:
    """A ``base_url`` argument is used even when the variable points elsewhere."""
    fixture_server.route("/quote-short", load_fixture("quote_short.json"))
    monkeypatch.setenv("FMP_BASE_URL", "https://unused.example/fmp")
    client = FmpClient(base_url=fixture_server.base_url, path_prefix="", auth_mode="none")
    assert len(client.quote.short("AAPL")) == 1
    assert fixture_server.requests[0].path == "/quote-short"


def test_auth_mode_none_with_the_env_base_sends_no_key(
    fixture_server: FixtureServer, monkeypatch: pytest.MonkeyPatch
) -> None:
    """``auth_mode="none"`` still ignores ``FMP_API_KEY`` when the base comes from the variable."""
    fixture_server.route("/stable/quote-short", load_fixture("quote_short.json"))
    monkeypatch.setenv("FMP_API_KEY", "vk_example")
    monkeypatch.setenv("FMP_BASE_URL", fixture_server.base_url)
    assert len(FmpClient(auth_mode="none").quote.short("AAPL")) == 1
    request = fixture_server.requests[0]
    assert "apikey" not in request.headers
    assert "apikey" not in request.query


@pytest.mark.parametrize("value", [None, "", "  \t\n"], ids=["unset", "empty", "blank"])
def test_unset_or_blank_env_base_keeps_the_default_host(
    errors: SimpleNamespace, monkeypatch: pytest.MonkeyPatch, value: str | None
) -> None:
    """Without a usable variable the client targets the default host, which needs a key."""
    if value is not None:
        monkeypatch.setenv("FMP_BASE_URL", value)
    with pytest.raises(errors.FmpConfigError) as raised:
        FmpClient()
    assert str(raised.value) == MISSING_TOKEN_MESSAGE


def test_plaintext_env_base_with_a_key_needs_the_danger_flag(
    errors: SimpleNamespace, monkeypatch: pytest.MonkeyPatch
) -> None:
    """A non-loopback ``http://`` base from the variable keeps the insecure-auth guard."""
    monkeypatch.setenv("FMP_API_KEY", "vk_example")
    monkeypatch.setenv("FMP_BASE_URL", "http://example.test")
    with pytest.raises(errors.FmpConfigError) as raised:
        FmpClient()
    assert str(raised.value) == INSECURE_MESSAGE
    assert "vk_example" not in repr(raised.value)
    FmpClient(danger_allow_insecure_authentication=True)


@pytest.mark.parametrize(
    "value", ["valet.bitbrew.app/fmp", "ftp://valet.bitbrew.app/fmp", "https://user:pw@valet.bitbrew.app/fmp"]
)
def test_invalid_env_base_raises_a_config_error(
    errors: SimpleNamespace, monkeypatch: pytest.MonkeyPatch, value: str
) -> None:
    """An unusable variable fails construction without echoing the value."""
    monkeypatch.setenv("FMP_API_KEY", "vk_example")
    monkeypatch.setenv("FMP_BASE_URL", value)
    with pytest.raises(errors.FmpConfigError) as raised:
        FmpClient()
    assert raised.value.category == "configuration"
    assert value not in str(raised.value)
