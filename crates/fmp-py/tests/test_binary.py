"""Runtime contract of ``BinaryPayload``, the result type of binary endpoints.

No registered namespace returns a binary payload yet (the ``statements``
domain lands with its own tests), so these tests build the payload from
Python exactly as the generated ``binary = true`` methods do from Rust.
"""

import ast
import pickle
import sys
from importlib.resources import files
from pathlib import Path

import pytest
from fmp import BinaryPayload

XLSX = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
WORKBOOK = b"PK\x03\x04\xff\xfe\x80\x00 not utf-8"


def test_payload_exposes_bytes_and_metadata() -> None:
    """``data`` is a real ``bytes`` object; the metadata round-trips verbatim."""
    payload = BinaryPayload(b"\x00\x01", "application/octet-stream")

    assert payload.data == b"\x00\x01"
    assert type(payload.data) is bytes
    assert payload.content_type == "application/octet-stream"
    assert payload.content_disposition is None
    assert payload.byte_len == 2
    assert len(payload) == 2


def test_payload_keeps_a_non_utf8_workbook_and_its_disposition() -> None:
    """Opaque XLSX bytes and the attachment header survive unchanged."""
    payload = BinaryPayload(WORKBOOK, XLSX, "attachment; filename=AAPL-2024-FY.xlsx")

    assert payload.data == WORKBOOK
    assert payload.byte_len == len(WORKBOOK)
    assert payload.content_type == XLSX
    assert payload.content_disposition == "attachment; filename=AAPL-2024-FY.xlsx"


def test_payload_is_frozen_and_picklable() -> None:
    """Attributes are read-only and a pickle round-trip restores every field."""
    payload = BinaryPayload(WORKBOOK, f"{XLSX}; source=provider", "attachment; filename=x.xlsx")

    with pytest.raises(AttributeError):
        payload.data = b""  # type: ignore[misc]
    restored = pickle.loads(pickle.dumps(payload))
    assert type(restored) is BinaryPayload
    assert restored.data == WORKBOOK
    assert restored.content_type == f"{XLSX}; source=provider"
    assert restored.content_disposition == "attachment; filename=x.xlsx"


def test_repr_shows_size_and_media_type_only() -> None:
    """The repr names the byte count and media type, never the body or disposition."""
    payload = BinaryPayload(WORKBOOK, f"{XLSX}; source=provider", "attachment; filename=secret.xlsx")

    assert repr(payload) == f"BinaryPayload(byte_len={len(WORKBOOK)}, content_type='{XLSX}')"
    assert "secret" not in repr(payload)


def test_constructor_requires_bytes() -> None:
    """A ``str`` body is rejected instead of being encoded silently."""
    with pytest.raises(TypeError):
        BinaryPayload("text", "text/plain")  # type: ignore[arg-type]


def test_payload_is_the_native_type_reexported_at_top_level() -> None:
    """``fmp.BinaryPayload`` is the ``fmp._native`` class under its native module path."""
    import fmp
    from fmp._native import BinaryPayload as native_payload

    assert BinaryPayload is native_payload
    assert "BinaryPayload" in fmp.__all__
    assert BinaryPayload.__module__ == "fmp._native"
    assert sys.modules["fmp._native"].BinaryPayload is BinaryPayload


def _stub_class(name: str) -> ast.ClassDef:
    stub = Path(str(files("fmp"))) / "_native" / "__init__.pyi"
    tree = ast.parse(stub.read_text(encoding="utf-8"), filename=str(stub))
    for node in tree.body:
        if isinstance(node, ast.ClassDef) and node.name == name:
            return node
    raise AssertionError(f"class {name} is not in {stub}")


def test_stub_types_data_as_bytes() -> None:
    """The shipped stub annotates ``data`` as ``bytes`` and ``byte_len`` as ``int``."""
    klass = _stub_class("BinaryPayload")
    returns = {
        node.name: ast.unparse(node.returns)
        for node in klass.body
        if isinstance(node, ast.FunctionDef) and node.returns is not None
    }

    assert returns["data"] == "bytes"
    assert returns["byte_len"] == "builtins.int"
    assert returns["content_type"] == "builtins.str"
    assert returns["content_disposition"] == "typing.Optional[builtins.str]"
    new = next(node for node in klass.body if isinstance(node, ast.FunctionDef) and node.name == "__new__")
    assert [ast.unparse(arg.annotation) for arg in new.args.args[1:]] == [
        "bytes",
        "builtins.str",
        "typing.Optional[builtins.str]",
    ]
