"""Client lifecycle and thread fan-out.

``FmpClient`` is synchronous, but every call releases the GIL while the Rust
transport waits, so one shared client can serve a thread pool and the requests
overlap. Run with ``uv run lifecycle_and_threads.py``; needs ``FMP_API_KEY``.
"""

# standard library
import time
from concurrent.futures import ThreadPoolExecutor

# fmp library
from fmp import FmpClient, FmpConfigError

SYMBOLS = ["AAPL", "MSFT", "NVDA", "AMZN", "GOOGL", "META"]


def fan_out(client: FmpClient) -> None:
    """Fetches one profile per symbol, sequentially and then on a thread pool."""
    started = time.perf_counter()
    for symbol in SYMBOLS:
        client.company.profile(symbol)
    sequential = time.perf_counter() - started

    started = time.perf_counter()
    # Share one client across threads: it pools connections internally. Keep
    # max_workers modest so a burst stays inside your FMP plan's rate limit.
    with ThreadPoolExecutor(max_workers=4) as pool:
        profiles = list(pool.map(lambda symbol: client.company.profile(symbol)[0], SYMBOLS))
    threaded = time.perf_counter() - started

    for profile in profiles:
        print(f"  {profile.symbol}: {profile.company_name}")
    print(f"  sequential {sequential:.2f}s, 4 threads {threaded:.2f}s")


def close_semantics() -> None:
    """Shows that a closed client, and every namespace taken from it, rejects calls."""
    client = FmpClient()
    quote = client.quote
    client.close()
    client.close()
    try:
        quote.short("AAPL")
    except FmpConfigError as error:
        print(f"  after close(), even via a saved namespace: {error}")


def main() -> None:
    """Uses the client as a context manager, then demonstrates close()."""
    # The with block closes the client on exit, releasing pooled connections; it
    # never swallows an exception raised inside the block.
    with FmpClient() as client:
        fan_out(client)
    close_semantics()


if __name__ == "__main__":
    main()
