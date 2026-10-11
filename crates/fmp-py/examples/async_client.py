"""The asyncio client: awaited calls, gather fan-out, and cancellation.

``AsyncFmpClient`` takes the same keywords as ``FmpClient`` and exposes the
same namespaces, but every endpoint method is a coroutine. Requests run on the
binding's own runtime, so awaiting one never blocks the event loop. Run with
``uv run async_client.py``; needs ``FMP_API_KEY`` (the cancellation part is
offline).
"""

# standard library
import asyncio
import time

# fmp library
from fmp import AsyncFmpClient

# local
from _local_server import serve  # isort: skip

SYMBOLS = ["AAPL", "MSFT", "NVDA", "AMZN", "GOOGL", "META"]


async def fan_out(client: AsyncFmpClient) -> None:
    """Fetches one profile per symbol, awaited one by one and then gathered."""
    started = time.perf_counter()
    for symbol in SYMBOLS:
        await client.company.profile(symbol)
    sequential = time.perf_counter() - started

    started = time.perf_counter()
    # gather overlaps the requests on one shared client. A semaphore keeps a
    # large fan-out inside your FMP plan's rate limit.
    limit = asyncio.Semaphore(4)

    async def profile(symbol: str) -> str:
        async with limit:
            rows = await client.company.profile(symbol)
        return f"{rows[0].symbol}: {rows[0].company_name}"

    lines = await asyncio.gather(*(profile(symbol) for symbol in SYMBOLS))
    gathered = time.perf_counter() - started

    for line in lines:
        print(f"  {line}")
    print(f"  sequential {sequential:.2f}s, gathered {gathered:.2f}s")


async def cancellation() -> None:
    """Shows that a timeout cancels the coroutine and aborts its HTTP request."""
    with serve(b"[]", delay=5.0) as base_url:
        async with AsyncFmpClient(base_url=base_url, path_prefix="", auth_mode="none") as client:
            started = time.perf_counter()
            try:
                await asyncio.wait_for(client.quote.short("AAPL"), timeout=0.5)
            except asyncio.TimeoutError:
                print(f"  gave up after {time.perf_counter() - started:.2f}s; the request was aborted")


async def main() -> None:
    """Uses the client as an async context manager, then demonstrates cancellation."""
    # async with closes the client on exit, releasing pooled connections; it
    # never swallows an exception raised inside the block.
    async with AsyncFmpClient() as client:
        await fan_out(client)
    await cancellation()


if __name__ == "__main__":
    asyncio.run(main())
