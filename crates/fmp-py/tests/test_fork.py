"""One client across Python threads and across ``os.fork``."""

import os
import signal
import threading
import time
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture


def test_one_client_supports_concurrent_python_threads(client: Any, fixture_server: FixtureServer) -> None:
    """Four threads share one client and every call completes."""
    fixture_server.route("/quote-short", load_fixture("quote_short.json"))
    barrier = threading.Barrier(5)
    failures: list[BaseException] = []

    def call_repeatedly() -> None:
        try:
            barrier.wait(timeout=5)
            for _ in range(5):
                assert client.quote.short("AAPL")[0].symbol == "AAPL"
        except BaseException as error:
            failures.append(error)

    threads = [threading.Thread(target=call_repeatedly) for _ in range(4)]
    for thread in threads:
        thread.start()
    barrier.wait(timeout=5)
    for thread in threads:
        thread.join(timeout=10)

    assert all(not thread.is_alive() for thread in threads)
    assert failures == []
    assert len(fixture_server.requests) == 20


@pytest.mark.skipif(not hasattr(os, "fork"), reason="os.fork is unavailable on this platform")
@pytest.mark.filterwarnings("ignore:This process .* is multi-threaded:DeprecationWarning")
def test_client_initialized_in_parent_remains_usable_after_fork(client: Any, fixture_server: FixtureServer) -> None:
    """A client that already made a request keeps working in a forked child.

    The fixture server's thread makes this process multi-threaded on purpose, so
    Python 3.12+ warns about ``fork``; that warning is the scenario under test.
    """
    fixture_server.route("/quote-short", load_fixture("quote_short.json"))
    assert client.quote.short("AAPL")[0].symbol == "AAPL"

    read_fd, write_fd = os.pipe()
    child_pid = os.fork()
    if child_pid == 0:
        os.close(read_fd)
        try:
            payload = client.quote.short("AAPL")[0].symbol.encode()
            exit_code = 0
        except BaseException as error:
            payload = f"{type(error).__name__}: {error}".encode()
            exit_code = 1
        os.write(write_fd, payload)
        os.close(write_fd)
        os._exit(exit_code)

    os.close(write_fd)
    deadline = time.monotonic() + 10
    while time.monotonic() < deadline:
        waited_pid, status = os.waitpid(child_pid, os.WNOHANG)
        if waited_pid == child_pid:
            break
        time.sleep(0.05)
    else:
        os.kill(child_pid, signal.SIGKILL)
        os.waitpid(child_pid, 0)
        raise AssertionError("the post-fork request did not complete within 10 seconds")

    payload = os.read(read_fd, 65_536)
    os.close(read_fd)
    assert os.WIFEXITED(status), f"child terminated abnormally with status {status}"
    assert os.WEXITSTATUS(status) == 0, payload.decode(errors="replace")
    assert payload == b"AAPL"
