"""Binary bodies and plain dicts: save an XLSX report, hand rows to other tools.

``statements.reports.xlsx`` returns one ``fmp.BinaryPayload`` (not a list): raw
``bytes`` plus the validated ``content_type``. Every typed row converts to a
plain ``dict`` with ``to_dict()``. Run with ``uv run binary_and_dicts.py``;
needs ``FMP_API_KEY``. The workbook goes to a temp dir.
"""

# standard library
import json
import tempfile
from pathlib import Path

# fmp library
from fmp import FmpClient, FmpDecodeError


def save_xlsx(client: FmpClient) -> None:
    """Downloads one annual report workbook and writes the bytes to disk."""
    try:
        payload = client.statements.reports.xlsx("AAPL", 2024, "FY")
    except FmpDecodeError as error:
        # A body whose Content-Type is not an XLSX (or generic binary) type is
        # rejected as a decode error. For a binary endpoint error.body is raw bytes
        # rendered as text, so log the structured attributes, not the body.
        print(f"xlsx rejected: category={error.category} status={error.status} endpoint={error.endpoint}")
        return
    print(f"{payload!r}: content_type={payload.content_type} bytes={payload.byte_len}")
    # content_disposition is the provider's filename hint, when it sent one. It is
    # untrusted text: never use it as a path without sanitizing it.
    print(f"content_disposition={payload.content_disposition}")
    path = Path(tempfile.mkdtemp(prefix="fmp-xlsx-")) / "AAPL-2024-FY.xlsx"
    path.write_bytes(payload.data)
    print(f"wrote {path}")


def rows_as_dicts(client: FmpClient) -> None:
    """Turns typed rows into dicts for JSON, a DataFrame, or any other consumer."""
    rows = client.statements.income.statement("AAPL", period="quarter", limit=4)
    records = [row.to_dict() for row in rows]
    print(f"{len(records)} records, {len(records[0])} columns, e.g. {list(records[0])[:5]}")
    # Dates stay datetime.date objects; default=str renders them for JSON.
    print(json.dumps(records[0], default=str)[:120] + "...")
    # pandas is not a dependency of these examples; with it installed:
    #     frame = pandas.DataFrame.from_records(records, index="date")


def main() -> None:
    """Runs the binary download and the dict hand-off."""
    with FmpClient() as client:
        save_xlsx(client)
        rows_as_dicts(client)


if __name__ == "__main__":
    main()
