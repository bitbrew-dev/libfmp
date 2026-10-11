"""Issue #422: ``AsyncFmpClient``, the asyncio twin of ``FmpClient``.

Each test drives its coroutines with ``asyncio.run`` inside a plain test
function, so the suite needs no async pytest plugin.
"""

import asyncio
import os
import signal
import time
from collections.abc import Callable, Coroutine
from types import SimpleNamespace
from typing import Any, TypeVar

import pytest
from conftest import CSV_CONTENT_TYPE, FixtureServer, load_csv_fixture, load_fixture
from fmp import AsyncFmpClient, BinaryPayload, FmpClient
from fmp.quote import AsyncQuoteNamespace, Quote
from fmp.statements import AsyncStatementsNamespace
from fmp.statements.income import AsyncStatementsIncomeNamespace

T = TypeVar("T")

XLSX = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
WORKBOOK = b"PK\x03\x04\xff\xfe\x80\x00 not utf-8"
QUOTE_SHORT_ROW = {"symbol": "AAPL", "price": 232.8, "change": 2.1, "volume": 44000000}


def run(coroutine: Coroutine[Any, Any, T]) -> T:
    """Run ``coroutine`` to completion on a fresh event loop."""
    return asyncio.run(coroutine)


def wait_until(condition: Callable[[], bool], timeout: float = 5.0) -> bool:
    """Poll ``condition`` until it holds or ``timeout`` seconds pass."""
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if condition():
            return True
        time.sleep(0.02)
    return condition()


def public_names(obj: Any) -> set[str]:
    """The public attribute names of ``obj``'s type."""
    return {name for name in dir(type(obj)) if not name.startswith("_")}


def test_endpoint_methods_are_coroutines_returning_models(async_client: Any, fixture_server: FixtureServer) -> None:
    """``await client.quote.full`` sends the same request and returns the same models as the sync client."""
    fixture_server.route("/quote", load_fixture("quote.json"))
    coroutine = async_client.quote.full("AAPL")
    assert asyncio.iscoroutine(coroutine)

    rows = run(coroutine)

    assert fixture_server.requests[0].target == "/quote?symbol=AAPL"
    assert isinstance(rows[0], Quote)
    assert rows[0].symbol == "AAPL"


def test_namespaces_are_the_async_generated_types(async_client: Any) -> None:
    """Domain getters return the ``Async*Namespace`` classes, nested ones included."""
    assert isinstance(async_client.quote, AsyncQuoteNamespace)
    assert isinstance(async_client.statements, AsyncStatementsNamespace)
    assert isinstance(async_client.statements.income, AsyncStatementsIncomeNamespace)


def test_every_sync_method_has_an_async_twin(client: Any, async_client: Any) -> None:
    """Both clients expose the same namespaces and methods, at every depth."""

    def compare(sync: Any, asynchronous: Any, path: str) -> int:
        assert public_names(asynchronous) - {"aclose"} == public_names(sync), path
        compared = 0
        for name in sorted(public_names(sync)):
            if name == "close":
                continue
            sync_member = getattr(sync, name)
            async_member = getattr(asynchronous, name)
            if callable(sync_member):
                assert callable(async_member), f"{path}.{name}"
                compared += 1
            else:
                compared += compare(sync_member, async_member, f"{path}.{name}")
        return compared

    assert compare(client, async_client, "client") > 250


def test_constructor_signature_matches_the_sync_client() -> None:
    """``AsyncFmpClient`` takes exactly the keywords ``FmpClient`` takes."""
    assert AsyncFmpClient.__text_signature__ == FmpClient.__text_signature__


def test_missing_credential_raises_the_sync_config_error(errors: SimpleNamespace) -> None:
    """Without a token or ``FMP_API_KEY`` the default host is rejected, naming the variable."""
    with pytest.raises(errors.FmpConfigError, match="FMP_API_KEY"):
        AsyncFmpClient()


def test_gather_runs_requests_concurrently(async_client: Any, fixture_server: FixtureServer) -> None:
    """Ten gathered calls against a 0.5 second route finish in about one route delay."""
    fixture_server.route("/quote-short", [QUOTE_SHORT_ROW], delay=0.5)

    async def fan_out() -> list[Any]:
        return await asyncio.gather(*(async_client.quote.short("AAPL") for _ in range(10)))

    started = time.monotonic()
    results = run(fan_out())
    elapsed = time.monotonic() - started

    assert [rows[0].symbol for rows in results] == ["AAPL"] * 10
    assert len(fixture_server.requests) == 10
    assert elapsed < 2.5, f"10 gathered calls took {elapsed:.2f}s; sequential would take 5s"


def test_event_loop_stays_responsive_while_a_request_is_in_flight(
    async_client: Any, fixture_server: FixtureServer
) -> None:
    """A ticker coroutine keeps running while one slow request is awaited."""
    fixture_server.route("/quote-short", [QUOTE_SHORT_ROW], delay=0.5)

    async def scenario() -> int:
        ticks = 0

        async def tick() -> None:
            nonlocal ticks
            while True:
                await asyncio.sleep(0.01)
                ticks += 1

        ticker = asyncio.ensure_future(tick())
        await async_client.quote.short("AAPL")
        ticker.cancel()
        return ticks

    assert run(scenario()) >= 10


def test_cancellation_aborts_the_in_flight_request(async_client: Any, fixture_server: FixtureServer) -> None:
    """Cancelling the awaiting task raises ``CancelledError`` at once and drops the connection."""
    fixture_server.route("/quote-short", [QUOTE_SHORT_ROW], delay=1.0)

    async def scenario() -> float:
        task = asyncio.ensure_future(async_client.quote.short("AAPL"))
        while not fixture_server.requests:
            await asyncio.sleep(0.01)
        task.cancel()
        started = time.monotonic()
        with pytest.raises(asyncio.CancelledError):
            await task
        return time.monotonic() - started

    assert run(scenario()) < 0.5
    assert wait_until(lambda: fixture_server.disconnects == 1), "the server never saw the client hang up"


def test_wait_for_timeout_cancels_like_a_task_cancel(async_client: Any, fixture_server: FixtureServer) -> None:
    """``asyncio.wait_for`` times out promptly and the request is aborted."""
    fixture_server.route("/quote-short", [QUOTE_SHORT_ROW], delay=1.0)

    async def scenario() -> None:
        await asyncio.wait_for(async_client.quote.short("AAPL"), timeout=0.2)

    with pytest.raises(asyncio.TimeoutError):
        run(scenario())
    assert wait_until(lambda: fixture_server.disconnects == 1)


def test_an_unawaited_coroutine_sends_nothing(async_client: Any, fixture_server: FixtureServer) -> None:
    """The request starts on the first ``await``, not when the method is called."""
    coroutine = async_client.quote.short("AAPL")
    coroutine.close()

    time.sleep(0.1)
    assert fixture_server.requests == []


def test_on_response_is_called_after_each_awaited_success(fixture_server: FixtureServer) -> None:
    """The callback sees the endpoint, status and allowlisted headers, once per call."""
    fixture_server.route("/quote-short", [QUOTE_SHORT_ROW], headers=(("X-Proxy-Cache", "HIT"),))
    calls: list[tuple[str, int, dict[str, str]]] = []

    def on_response(endpoint_id: str, status: int, headers: dict[str, str]) -> None:
        calls.append((endpoint_id, status, headers))

    client = AsyncFmpClient(base_url=fixture_server.base_url, path_prefix="", auth_mode="none", on_response=on_response)

    async def scenario() -> None:
        await asyncio.gather(client.quote.short("AAPL"), client.quote.short("MSFT"))

    run(scenario())
    assert calls == [("quote-short", 200, {"x-proxy-cache": "HIT"})] * 2


def test_on_response_exception_propagates_from_the_await(fixture_server: FixtureServer) -> None:
    """A raising callback makes the coroutine raise that exception instead of returning data."""
    fixture_server.route("/quote-short", [QUOTE_SHORT_ROW])

    def on_response(endpoint_id: str, status: int, headers: dict[str, str]) -> None:
        raise LookupError(f"rejected {endpoint_id}")

    client = AsyncFmpClient(base_url=fixture_server.base_url, path_prefix="", auth_mode="none", on_response=on_response)

    with pytest.raises(LookupError, match="rejected quote-short"):
        run(client.quote.short("AAPL"))


def test_errors_raise_the_sync_exception_classes(
    async_client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """Status, decode and validation failures surface as the same structured exceptions."""
    fixture_server.route("/quote-short", {"Error Message": "denied"}, status=401)
    fixture_server.route("/quote", b"not-json", content_type="text/plain")

    with pytest.raises(errors.FmpStatusError) as status:
        run(async_client.quote.short("AAPL"))
    assert status.value.status == 401
    with pytest.raises(errors.FmpDecodeError):
        run(async_client.quote.full("AAPL"))
    with pytest.raises(errors.FmpValidationError):
        run(async_client.quote.full(""))
    assert len(fixture_server.requests) == 2


def test_closed_client_rejects_calls_through_existing_namespaces(
    async_client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """After ``close()`` every call raises ``FmpConfigError``, even via a namespace fetched before."""
    namespace = async_client.quote
    async_client.close()
    async_client.close()

    with pytest.raises(errors.FmpConfigError, match="closed"):
        run(namespace.short("AAPL"))
    assert fixture_server.requests == []


def test_aclose_is_awaitable_and_idempotent(async_client: Any, errors: SimpleNamespace) -> None:
    """``await aclose()`` closes the client; a second close is a no-op."""

    async def scenario() -> None:
        await async_client.aclose()
        await async_client.aclose()
        await async_client.quote.short("AAPL")

    with pytest.raises(errors.FmpConfigError, match="closed"):
        run(scenario())


def test_async_with_yields_the_client_and_closes_it(
    async_client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """``async with`` binds the client itself and closes it on exit, letting exceptions propagate."""
    fixture_server.route("/quote-short", [QUOTE_SHORT_ROW])

    async def scenario() -> None:
        async with async_client as bound:
            assert bound is async_client
            assert (await bound.quote.short("AAPL"))[0].symbol == "AAPL"
        with pytest.raises(errors.FmpConfigError, match="closed"):
            await async_client.quote.short("AAPL")

    run(scenario())

    async def raising() -> None:
        async with AsyncFmpClient(base_url=fixture_server.base_url, auth_mode="none"):
            raise KeyError("from the block")

    with pytest.raises(KeyError, match="from the block"):
        run(raising())


def test_close_lets_an_in_flight_call_finish(async_client: Any, fixture_server: FixtureServer) -> None:
    """A call already awaiting its response completes after ``close()``, like the sync client."""
    fixture_server.route("/quote-short", [QUOTE_SHORT_ROW], delay=0.3)

    async def scenario() -> Any:
        task = asyncio.ensure_future(async_client.quote.short("AAPL"))
        while not fixture_server.requests:
            await asyncio.sleep(0.01)
        await async_client.aclose()
        return await task

    assert run(scenario())[0].symbol == "AAPL"


def test_xlsx_endpoint_awaits_a_binary_payload(async_client: Any, fixture_server: FixtureServer) -> None:
    """``await statements.reports.xlsx`` returns the workbook bytes and media type."""
    fixture_server.route("/financial-reports-xlsx", WORKBOOK, content_type=XLSX)

    payload = run(async_client.statements.reports.xlsx("AAPL", 2022, "FY"))

    assert fixture_server.requests[0].target == "/financial-reports-xlsx?symbol=AAPL&year=2022&period=FY"
    assert isinstance(payload, BinaryPayload)
    assert payload.data == WORKBOOK
    assert payload.content_type == XLSX


def test_bulk_csv_endpoint_awaits_decoded_rows(async_client: Any, fixture_server: FixtureServer) -> None:
    """A bulk CSV method decodes its rows the same way as the sync client."""
    fixture_server.route("/profile-bulk", load_csv_fixture("bulk_company_profiles.csv"), content_type=CSV_CONTENT_TYPE)

    rows = run(async_client.bulk.company_profiles("0"))

    assert fixture_server.requests[0].target == "/profile-bulk?part=0"
    assert len(rows) == 4
    assert rows[0].symbol == "WMB"


def test_dynamic_rows_are_awaited_as_dicts(async_client: Any, fixture_server: FixtureServer) -> None:
    """An untyped-row endpoint returns plain ``dict`` rows from the coroutine."""
    fixture_server.route("/industry-classification-search", [{"symbol": "AAPL", "sicCode": "3571"}])

    rows = run(async_client.sec_filings.search_industry_classifications(symbol="AAPL"))

    assert rows == [{"symbol": "AAPL", "sicCode": "3571"}]


def test_sync_and_async_clients_share_one_process(
    client: Any, async_client: Any, fixture_server: FixtureServer
) -> None:
    """A sync call inside a coroutine and an awaited call both succeed on the shared runtime."""
    fixture_server.route("/quote-short", [QUOTE_SHORT_ROW])

    async def scenario() -> tuple[str, str]:
        sync_rows = client.quote.short("AAPL")
        async_rows = await async_client.quote.short("AAPL")
        return sync_rows[0].symbol, async_rows[0].symbol

    assert run(scenario()) == ("AAPL", "AAPL")


@pytest.mark.skipif(not hasattr(os, "fork"), reason="os.fork is unavailable on this platform")
@pytest.mark.filterwarnings("ignore:This process .* is multi-threaded:DeprecationWarning")
def test_async_client_used_in_parent_works_after_fork(async_client: Any, fixture_server: FixtureServer) -> None:
    """A forked child awaits on a fresh event loop with the client its parent already used."""
    fixture_server.route("/quote-short", [QUOTE_SHORT_ROW])
    assert run(async_client.quote.short("AAPL"))[0].symbol == "AAPL"

    read_fd, write_fd = os.pipe()
    child_pid = os.fork()
    if child_pid == 0:
        os.close(read_fd)
        try:
            payload = asyncio.run(async_client.quote.short("AAPL"))[0].symbol.encode()
            exit_code = 0
        except BaseException as error:
            payload = f"{type(error).__name__}: {error}".encode()
            exit_code = 1
        os.write(write_fd, payload)
        os.close(write_fd)
        os._exit(exit_code)

    os.close(write_fd)
    if not wait_until(lambda: os.waitpid(child_pid, os.WNOHANG)[0] == child_pid, timeout=10):
        os.kill(child_pid, signal.SIGKILL)
        os.waitpid(child_pid, 0)
        raise AssertionError("the post-fork await did not complete within 10 seconds")
    payload = os.read(read_fd, 65_536)
    os.close(read_fd)
    assert payload == b"AAPL", payload.decode(errors="replace")
