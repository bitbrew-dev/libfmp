"""Secret-URL contract for ``FinancialReportDate`` (#175).

``link_json`` / ``link_xlsx`` embed the API key. They are never plain
attributes: reading one takes an explicit ``expose_secret_url_*()`` call, and
``repr()`` / ``str()`` print ``[REDACTED URL]`` unless the process-wide reveal
flag is on. No network is involved: the models are built through ``__new__``.
"""

import os
import pickle
import subprocess
import sys
from collections.abc import Iterator
from typing import Any

import pytest

SECRET = "SECRET-TOKEN-0123"
LINK_JSON = f"https://example.test/report.json?symbol=AAPL&apikey={SECRET}"
LINK_XLSX = f"https://example.test/report.xlsx?symbol=AAPL&apikey={SECRET}"
REDACTED = "[REDACTED URL]"


@pytest.fixture
def reveal_off() -> Iterator[None]:
    """Force the reveal flag off for the test and restore it afterwards.

    The flag is process-global and seeded from ``FMP_REVEAL_SECRET_URLS`` at
    first use, so the fixture pins it at setup rather than trusting the
    environment, then puts back whatever it found.
    """
    from fmp import reveal_secret_urls, set_reveal_secret_urls

    previous = reveal_secret_urls()
    set_reveal_secret_urls(False)
    try:
        yield
    finally:
        set_reveal_secret_urls(previous)


@pytest.fixture
def report_date(reveal_off: None) -> Any:
    """A ``FinancialReportDate`` whose links carry the fake key."""
    from fmp.statements.reports import FinancialReportDate

    return FinancialReportDate(
        symbol="AAPL", fiscal_year=2024, period="FY", link_json=LINK_JSON, link_xlsx=LINK_XLSX
    )


def test_toggle_functions_are_exported_from_the_top_level_package() -> None:
    """``fmp`` re-exports the native toggle pair under the same objects."""
    import fmp
    from fmp import reveal_secret_urls, set_reveal_secret_urls
    from fmp._native import reveal_secret_urls as native_get, set_reveal_secret_urls as native_set

    assert {"reveal_secret_urls", "set_reveal_secret_urls"} <= set(fmp.__all__)
    assert reveal_secret_urls is native_get
    assert set_reveal_secret_urls is native_set


def test_repr_and_str_redact_by_default(report_date: Any) -> None:
    """Neither ``repr()`` nor ``str()`` contains the key while the flag is off."""
    expected = (
        f"FinancialReportDate(symbol='AAPL', fiscal_year=2024, period='FY', link_json={REDACTED}, link_xlsx={REDACTED})"
    )
    assert repr(report_date) == expected
    assert str(report_date) == expected
    assert SECRET not in repr(report_date)
    assert SECRET not in str(report_date)


def test_links_are_not_attributes(report_date: Any) -> None:
    """The URLs are reachable only through the explicit accessors."""
    assert not hasattr(report_date, "link_json")
    assert not hasattr(report_date, "link_xlsx")
    assert (report_date.symbol, report_date.fiscal_year, report_date.period) == ("AAPL", 2024, "FY")


def test_expose_accessors_return_the_full_urls(report_date: Any) -> None:
    """The conspicuous accessors hand back the URL including the key."""
    assert report_date.expose_secret_url_json() == LINK_JSON
    assert report_date.expose_secret_url_xlsx() == LINK_XLSX


def test_reveal_toggle_switches_repr(report_date: Any) -> None:
    """Turning the flag on reveals the URLs in ``repr()``; off redacts again."""
    from fmp import reveal_secret_urls, set_reveal_secret_urls

    assert reveal_secret_urls() is False
    set_reveal_secret_urls(True)
    assert reveal_secret_urls() is True
    assert repr(report_date) == (
        "FinancialReportDate(symbol='AAPL', fiscal_year=2024, period='FY', "
        f"link_json='{LINK_JSON}', link_xlsx='{LINK_XLSX}')"
    )
    assert SECRET in str(report_date)
    set_reveal_secret_urls(False)
    assert SECRET not in repr(report_date)


def test_repr_quotes_fields_like_python(reveal_off: None) -> None:
    """String fields are rendered with Python ``repr`` quoting rules."""
    from fmp.statements.reports import FinancialReportDate

    quoted = FinancialReportDate(
        symbol="A'B", fiscal_year=1999, period="Q1", link_json=LINK_JSON, link_xlsx=LINK_XLSX
    )
    assert repr(quoted).startswith("FinancialReportDate(symbol=\"A'B\", fiscal_year=1999, period='Q1', ")


def test_pickle_round_trip_keeps_the_urls(report_date: Any) -> None:
    """Pickling preserves the URLs, so the pickle bytes contain the key."""
    payload = pickle.dumps(report_date)
    restored = pickle.loads(payload)

    assert SECRET.encode() in payload
    assert type(restored) is type(report_date)
    assert restored.symbol == "AAPL"
    assert restored.expose_secret_url_json() == LINK_JSON
    assert restored.expose_secret_url_xlsx() == LINK_XLSX
    assert SECRET not in repr(restored)


def _reveal_flag_in_subprocess(value: str | None) -> bool:
    """Start a fresh interpreter with ``FMP_REVEAL_SECRET_URLS`` set to ``value``."""
    env = {name: item for name, item in os.environ.items() if name != "FMP_REVEAL_SECRET_URLS"}
    if value is not None:
        env["FMP_REVEAL_SECRET_URLS"] = value
    code = (
        "import fmp\n"
        "from fmp.statements.reports import FinancialReportDate\n"
        f"row = FinancialReportDate(symbol='AAPL', fiscal_year=2024, period='FY', link_json={LINK_JSON!r}, link_xlsx={LINK_XLSX!r})\n"
        f"print(fmp.reveal_secret_urls(), {SECRET!r} in repr(row))\n"
    )
    completed = subprocess.run(
        [sys.executable, "-c", code], env=env, capture_output=True, text=True, check=True, timeout=60
    )
    flag, revealed = completed.stdout.split()
    assert flag == revealed
    return flag == "True"


@pytest.mark.parametrize("value", ["1", "true", "YES", " True "])
def test_environment_variable_enables_revealing(value: str) -> None:
    """A truthy ``FMP_REVEAL_SECRET_URLS`` reveals URLs from the first repr."""
    assert _reveal_flag_in_subprocess(value) is True


@pytest.mark.parametrize("value", [None, "", "0", "false", "no", "on"])
def test_environment_variable_defaults_to_redaction(value: str | None) -> None:
    """Unset or non-truthy values keep the default redaction."""
    assert _reveal_flag_in_subprocess(value) is False
