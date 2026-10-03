"""Stub-drift gate: the shipped ``.pyi`` files must match the built extension.

``mypy.stubtest`` imports the installed ``fmp`` package and compares every
runtime object with its stub, so a stub that names a missing class, a wrong
signature, or an unresolvable annotation fails here. ``mypy`` also checks the
static contract in ``typecheck_contract.py`` and, in strict mode, the runnable
scripts under ``examples/`` against the same stubs.

Both tests need ``mypy`` in the test interpreter and skip without it. They run
from a temporary directory so ``fmp`` resolves to the installed package and the
mypy cache stays out of the tree.
"""

import subprocess
import sys
from pathlib import Path

import pytest

pytest.importorskip("mypy")

CONTRACT = Path(__file__).with_name("typecheck_contract.py")

EXAMPLES = Path(__file__).resolve().parents[1] / "examples"


def _mypy(tmp_path: Path, *arguments: str) -> subprocess.CompletedProcess[str]:
    """Runs ``python -m <arguments>`` from ``tmp_path`` and captures its output."""
    return subprocess.run(
        [sys.executable, "-m", *arguments],
        cwd=tmp_path,
        capture_output=True,
        text=True,
        check=False,
        timeout=600,
    )


def test_stubs_match_the_runtime_module(tmp_path: Path) -> None:
    """``stubtest`` finds no difference between the stubs and the runtime."""
    result = _mypy(tmp_path, "mypy.stubtest", "fmp")
    assert result.returncode == 0, result.stdout + result.stderr


def test_typecheck_contract_passes_mypy(tmp_path: Path) -> None:
    """The static contract type-checks cleanly against the shipped stubs."""
    result = _mypy(tmp_path, "mypy", "--cache-dir", str(tmp_path / "cache"), str(CONTRACT))
    assert result.returncode == 0, result.stdout + result.stderr


def test_examples_pass_strict_mypy(tmp_path: Path) -> None:
    """The ``examples/`` scripts type-check under ``--strict`` against the shipped stubs."""
    result = _mypy(tmp_path, "mypy", "--strict", "--cache-dir", str(tmp_path / "cache"), str(EXAMPLES))
    assert result.returncode == 0, result.stdout + result.stderr
