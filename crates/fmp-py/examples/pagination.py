"""Pagination: walk a paged endpoint with the keyword-only ``page`` and ``limit``.

Paged endpoints take ``page`` (0-based) and ``limit`` (rows per page) as
keyword-only arguments. Stop when a page comes back shorter than ``limit``, and
cap the walk so a large history cannot run away with your rate limit. Run with
``uv run pagination.py``; needs ``FMP_API_KEY``.
"""

# standard library
from collections import Counter
from collections.abc import Iterator

# fmp library
from fmp import FmpClient
from fmp.congressional import CongressionalTrade

PAGE_SIZE = 25
MAX_PAGES = 3


def senate_disclosures(client: FmpClient) -> Iterator[CongressionalTrade]:
    """Yields the latest Senate disclosures, one page at a time."""
    for page in range(MAX_PAGES):
        rows = client.congressional.latest_senate_disclosures(page=page, limit=PAGE_SIZE)
        print(f"page {page}: {len(rows)} rows")
        yield from rows
        if len(rows) < PAGE_SIZE:
            return


def main() -> None:
    """Collects a few pages and summarizes them."""
    with FmpClient() as client:
        trades = list(senate_disclosures(client))
    if not trades:
        return
    print(f"{len(trades)} trades, newest disclosure {max(trade.disclosure_date for trade in trades)}")
    for kind, count in Counter(trade.transaction_type for trade in trades).most_common():
        print(f"  {kind}: {count}")


if __name__ == "__main__":
    main()
