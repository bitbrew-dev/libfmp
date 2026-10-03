"""Bulk download: one call returns every row, then write it out with the csv module.

Bulk endpoints answer with a whole market in one body (often tens of MiB), so they
get longer defaults: 600 s and 256 MiB (see ``timeouts_and_limits.py``). This uses
``price_target_summaries``, one of the smallest. Run with ``uv run bulk.py``;
needs ``FMP_API_KEY`` and a plan with bulk access. The CSV goes to a temp dir.
"""

# standard library
import csv
import tempfile
from pathlib import Path

# fmp library
from fmp import FmpClient


def main() -> None:
    """Downloads the bulk snapshot and writes it as CSV."""
    # No timeout= here on purpose: an explicit value would also replace the 600 s
    # bulk default, and a large bulk body can take minutes on a slow link.
    with FmpClient() as client:
        rows = client.bulk.price_target_summaries()
    print(f"{len(rows)} rows")
    if not rows:
        return

    # to_dict() keys are the snake_case attribute names, in constructor order.
    records = [row.to_dict() for row in rows]
    path = Path(tempfile.mkdtemp(prefix="fmp-bulk-")) / "price_target_summaries.csv"
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(records[0]))
        writer.writeheader()
        writer.writerows(records)
    print(f"wrote {path} ({path.stat().st_size:,} bytes, columns: {', '.join(records[0])})")


if __name__ == "__main__":
    main()
