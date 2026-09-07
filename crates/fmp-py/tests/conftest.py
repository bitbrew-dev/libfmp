"""Shared pytest harness for the ``fmp`` Python bindings.

The harness has three parts:

* ``load_fixture(name)`` reads a JSON body from ``crates/libfmp/tests/fixtures/``,
  the same files the Rust ``*_responses.rs`` decode tests ``include_str!``.
* ``FixtureServer`` is a loopback HTTP server driven by a route table. Each
  route maps a request path to a status, a body, and a content type; requests
  are logged so tests can assert the wire shape (path, query, headers).
* ``errors`` bundles the exception types under their public names so the
  per-domain tests never import ``fmp._native`` directly.

Route lookup order for a request: the exact ``path?query`` string, then the
bare path without its query string, then the server's default body. Register
status and decode failures on the bare path (or on a full ``path?query`` key
that includes every parameter the client sends); query authentication appends
``apikey=...`` to the query string, so a symbol-only key would not match.

Per-domain template (a fan-out issue adds ``tests/test_<domain>.py``)::

    from conftest import FixtureServer, load_fixture

    def test_profile_decodes_documented_fixture(client, fixture_server):
        fixture_server.route("/profile", load_fixture("company_profile.json"))
        rows = client.company.profile("AAPL")
        assert fixture_server.requests[0].path == "/profile"
        assert fixture_server.requests[0].query["symbol"] == ["AAPL"]
        assert rows[0].symbol == "AAPL"

Cover every query-argument shape with at least one request-log assertion, and
one failure route (``status=401`` or ``body=b"not-json"``) per namespace to
prove the error mapping stays structured. The ``client`` fixture targets the
server with ``path_prefix=""`` and ``auth_mode="none"``; build your own
``FmpClient(base_url=fixture_server.base_url, ...)`` for other configurations.
"""

import json
import threading
from collections.abc import Iterator
from dataclasses import dataclass, field
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from types import SimpleNamespace
from typing import Any
from urllib.parse import parse_qs, urlsplit

import pytest

FIXTURES_DIR = Path(__file__).resolve().parents[2] / "libfmp" / "tests" / "fixtures"

JSON_CONTENT_TYPE = "application/json"


def load_fixture(name: str) -> Any:
    """Return the parsed JSON body of ``crates/libfmp/tests/fixtures/<name>``."""
    return json.loads((FIXTURES_DIR / name).read_text(encoding="utf-8"))


def _errors_namespace() -> SimpleNamespace:
    """Expose the exception hierarchy under its public names.

    The types come from the ``fmp.errors`` package (#172); the namespace keeps
    the fixture shape stable so per-domain tests never import ``fmp._native``.
    """
    from fmp.errors import (
        FmpConfigError,
        FmpDecodeError,
        FmpError,
        FmpStatusError,
        FmpTransportError,
        FmpValidationError,
    )

    return SimpleNamespace(
        FmpError=FmpError,
        FmpValidationError=FmpValidationError,
        FmpConfigError=FmpConfigError,
        FmpTransportError=FmpTransportError,
        FmpStatusError=FmpStatusError,
        FmpDecodeError=FmpDecodeError,
    )


@dataclass(frozen=True)
class Route:
    """One canned response: HTTP status, body, and content type.

    ``body`` is sent verbatim when it is ``bytes``; any other value is JSON
    encoded.
    """

    body: Any = field(default_factory=list)
    status: int = 200
    content_type: str = JSON_CONTENT_TYPE

    def payload(self) -> bytes:
        """Return the encoded response body."""
        if isinstance(self.body, bytes):
            return self.body
        return json.dumps(self.body).encode("utf-8")


@dataclass(frozen=True)
class RecordedRequest:
    """One request the fixture server received."""

    path: str
    raw_query: str
    query: dict[str, list[str]]
    headers: dict[str, str]
    connection: tuple[str, int]

    @property
    def target(self) -> str:
        """The request target as sent: the path plus any query string."""
        return f"{self.path}?{self.raw_query}" if self.raw_query else self.path


class FixtureServer:
    """A loopback HTTP server that answers from a route table.

    ``routes`` maps a request path (or a full ``path?query`` string) to a
    :class:`Route`; unmatched requests receive ``default``. Every request is
    appended to ``requests`` in arrival order.
    """

    def __init__(self, default: Route | None = None) -> None:
        """Create a server bound to an ephemeral loopback port; call ``start``."""
        self.routes: dict[str, Route] = {}
        self.default: Route = default if default is not None else Route()
        self.requests: list[RecordedRequest] = []
        self._server = ThreadingHTTPServer(("127.0.0.1", 0), self._handler_class())
        self._thread = threading.Thread(target=self._server.serve_forever, daemon=True)

    @property
    def base_url(self) -> str:
        """The ``http://127.0.0.1:<port>`` origin of this server."""
        return f"http://127.0.0.1:{self._server.server_port}"

    def route(
        self,
        path: str,
        body: Any = None,
        *,
        status: int = 200,
        content_type: str = JSON_CONTENT_TYPE,
    ) -> Route:
        """Register the response for ``path`` and return the stored route."""
        stored = Route(body=[] if body is None else body, status=status, content_type=content_type)
        self.routes[path] = stored
        return stored

    def lookup(self, raw_target: str) -> Route:
        """Resolve ``raw_target`` (``path?query``) against the route table."""
        parts = urlsplit(raw_target)
        return self.routes.get(raw_target) or self.routes.get(parts.path) or self.default

    def start(self) -> None:
        """Begin serving on the background thread."""
        self._thread.start()

    def stop(self) -> None:
        """Shut the server down and join its thread."""
        self._server.shutdown()
        self._server.server_close()
        self._thread.join(timeout=5)

    def _record(self, handler: BaseHTTPRequestHandler) -> None:
        parts = urlsplit(handler.path)
        headers = {name.lower(): value for name, value in handler.headers.items()}
        self.requests.append(
            RecordedRequest(
                path=parts.path,
                raw_query=parts.query,
                query=parse_qs(parts.query),
                headers=headers,
                connection=handler.client_address,
            )
        )

    def _handler_class(self) -> type[BaseHTTPRequestHandler]:
        server = self

        class Handler(BaseHTTPRequestHandler):
            """Serve the route table; keep-alive so connection reuse is observable."""

            protocol_version = "HTTP/1.1"

            def do_GET(self) -> None:
                """Answer a GET from the route table."""
                server._record(self)
                route = server.lookup(self.path)
                payload = route.payload()
                self.send_response(route.status)
                self.send_header("Content-Type", route.content_type)
                self.send_header("Content-Length", str(len(payload)))
                self.end_headers()
                self.wfile.write(payload)

            def log_message(self, _format: str, *_args: Any) -> None:
                """Silence the default stderr access log."""

        return Handler


@pytest.fixture
def fixture_server() -> Iterator[FixtureServer]:
    """A running :class:`FixtureServer` with an empty route table."""
    server = FixtureServer()
    server.start()
    try:
        yield server
    finally:
        server.stop()


@pytest.fixture
def client(fixture_server: FixtureServer) -> Any:
    """An unauthenticated ``FmpClient`` pointed at ``fixture_server``."""
    from fmp import FmpClient

    return FmpClient(base_url=fixture_server.base_url, path_prefix="", auth_mode="none")


@pytest.fixture
def errors() -> SimpleNamespace:
    """The exception hierarchy under its public names (see ``_errors_namespace``)."""
    return _errors_namespace()
