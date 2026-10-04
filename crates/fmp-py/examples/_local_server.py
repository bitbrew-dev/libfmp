"""A tiny loopback HTTP server the offline examples point ``FmpClient`` at.

It answers every request with one canned response, so ``errors.py`` and
``timeouts_and_limits.py`` can show each failure mode without a key, without
network access, and without depending on what the real provider sends today.
"""

# standard library
import threading
import time
from collections.abc import Iterator
from contextlib import contextmanager
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Any


@contextmanager
def serve(
    body: bytes, *, status: int = 200, content_type: str = "application/json", delay: float = 0.0
) -> Iterator[str]:
    """Serves ``body`` for every GET until the block exits; yields the origin URL."""

    class Handler(BaseHTTPRequestHandler):
        """Answers every GET with the canned response."""

        def do_GET(self) -> None:
            """Waits ``delay`` seconds, then sends the canned response.

            A client that gave up (a timeout or a size limit) has already closed
            the socket, so a failed write is expected and ignored.
            """
            time.sleep(delay)
            try:
                self.send_response(status)
                self.send_header("Content-Type", content_type)
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)
            except (BrokenPipeError, ConnectionResetError):
                pass

        def log_message(self, format: str, *args: Any) -> None:
            """Silences the default stderr access log."""

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    try:
        yield f"http://127.0.0.1:{server.server_port}"
    finally:
        server.shutdown()
        server.server_close()
        thread.join(timeout=5)
