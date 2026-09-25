"""Runtime contract of ``client.calendar`` for dividends, earnings, and splits.

One test per method routes the documented fixture body, calls the method with
one argument shape, and asserts the exact request target plus a few typed
fields (including the ``datetime.date`` ones). The expected targets are the
ones the Rust ``calendar_*_endpoints.rs`` tests pin. The IPO methods and the
negative cases live in ``test_calendar_b.py``.
"""

import datetime
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp.calendar import CalendarNamespace, DividendEvent, EarningsEvent, StockSplitEvent

FROM_2026_01_27 = datetime.date(2026, 1, 27)
TO_2026_04_27 = datetime.date(2026, 4, 27)


def test_calendar_namespace_is_the_generated_type(client: Any) -> None:
    """``client.calendar`` is the generated flat namespace class."""
    assert isinstance(client.calendar, CalendarNamespace)


def test_dividends_with_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``dividends`` encodes ``symbol`` then ``limit`` and exposes ``yield`` as ``yield_``."""
    fixture_server.route("/dividends", load_fixture("dividends.json"))
    rows = client.calendar.dividends("AAPL", limit=1001)

    assert fixture_server.requests[0].target == "/dividends?symbol=AAPL&limit=1001"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, DividendEvent)
    assert row.symbol == "AAPL"
    assert row.date == datetime.date(2026, 5, 11)
    assert row.record_date == datetime.date(2026, 5, 11)
    assert row.payment_date == datetime.date(2026, 5, 14)
    assert row.declaration_date == datetime.date(2026, 4, 30)
    assert row.dividend == pytest.approx(0.27)
    assert row.yield_ == pytest.approx(0.3587535875358754)
    assert row.frequency == "Quarterly"
    assert not hasattr(row, "yield")


def test_dividends_without_options_sends_only_the_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``dividends`` omits ``limit`` when it is not given."""
    fixture_server.route("/dividends", load_fixture("dividends.json"))
    rows = client.calendar.dividends("AAPL")

    assert fixture_server.requests[0].target == "/dividends?symbol=AAPL"
    assert len(rows) == 1


def test_dividends_calendar_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``dividends_calendar`` takes ``from_`` and ``to`` as dates and encodes ``from``, ``to``, ``page``."""
    fixture_server.route("/dividends-calendar", load_fixture("dividends_calendar.json"))
    rows = client.calendar.dividends_calendar(from_=FROM_2026_01_27, to=TO_2026_04_27, page=0)

    assert fixture_server.requests[0].target == "/dividends-calendar?from=2026-01-27&to=2026-04-27&page=0"
    assert fixture_server.requests[0].query["from"] == ["2026-01-27"]
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, DividendEvent)
    assert row.symbol == "5871.TW"
    assert row.date == datetime.date(2026, 7, 29)
    assert row.payment_date == datetime.date(2026, 9, 1)
    assert row.declaration_date is None
    assert row.adj_dividend == pytest.approx(5.8)
    assert row.yield_ == pytest.approx(5.155555555555556)
    assert row.frequency == "Annual"


def test_dividends_calendar_omits_to_independently(client: Any, fixture_server: FixtureServer) -> None:
    """``dividends_calendar`` accepts an ISO string for ``from_`` and leaves ``to`` out."""
    fixture_server.route("/dividends-calendar", load_fixture("dividends_calendar.json"))
    rows = client.calendar.dividends_calendar(from_="2026-01-27", page=0)

    assert fixture_server.requests[0].target == "/dividends-calendar?from=2026-01-27&page=0"
    assert len(rows) == 1


def test_earnings_with_limit_and_explicit_false_flag(client: Any, fixture_server: FixtureServer) -> None:
    """``earnings`` encodes ``symbol``, ``limit``, then ``includeReportTimes=false``."""
    fixture_server.route("/earnings", load_fixture("earnings.json"))
    rows = client.calendar.earnings("AAPL", limit=4_294_967_295, include_report_times=False)

    assert fixture_server.requests[0].target == "/earnings?symbol=AAPL&limit=4294967295&includeReportTimes=false"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EarningsEvent)
    assert row.symbol == "AAPL"
    assert row.date == datetime.date(2026, 7, 30)
    assert row.last_updated == datetime.date(2026, 7, 30)
    assert row.eps_actual is None
    assert row.eps_estimated == pytest.approx(1.88)
    assert row.revenue_actual is None
    assert row.revenue_estimated == 109_038_900_000


def test_earnings_with_true_flag_only(client: Any, fixture_server: FixtureServer) -> None:
    """``earnings`` omits ``limit`` and still sends ``includeReportTimes=true``."""
    fixture_server.route("/earnings", load_fixture("earnings.json"))
    rows = client.calendar.earnings("AAPL", include_report_times=True)

    assert fixture_server.requests[0].target == "/earnings?symbol=AAPL&includeReportTimes=true"
    assert len(rows) == 1


def test_earnings_calendar_without_options_has_no_query_string(client: Any, fixture_server: FixtureServer) -> None:
    """``earnings_calendar`` with nothing set requests the bare path."""
    fixture_server.route("/earnings-calendar", load_fixture("earnings_calendar.json"))
    rows = client.calendar.earnings_calendar()

    assert fixture_server.requests[0].target == "/earnings-calendar"
    assert fixture_server.requests[0].raw_query == ""
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EarningsEvent)
    assert row.symbol == "GRG.L"
    assert row.date == datetime.date(2026, 7, 29)
    assert row.eps_actual == pytest.approx(0.549)
    assert row.revenue_actual == 1_101_500_000
    assert row.revenue_estimated == 1_086_300_000


def test_earnings_calendar_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``earnings_calendar`` encodes ``from``, ``to``, ``page``, ``includeReportTimes`` in that order."""
    fixture_server.route("/earnings-calendar", load_fixture("earnings_calendar.json"))
    rows = client.calendar.earnings_calendar(
        from_=datetime.date(2026, 4, 27), to="2026-07-26", page=0, include_report_times=True
    )

    assert (
        fixture_server.requests[0].target
        == "/earnings-calendar?from=2026-04-27&to=2026-07-26&page=0&includeReportTimes=true"
    )
    assert len(rows) == 1
    assert rows[0].last_updated == datetime.date(2026, 7, 30)


def test_earnings_calendar_with_to_page_and_false_flag(client: Any, fixture_server: FixtureServer) -> None:
    """``earnings_calendar`` omits ``from`` independently and sends ``includeReportTimes=false``."""
    fixture_server.route("/earnings-calendar", load_fixture("earnings_calendar.json"))
    rows = client.calendar.earnings_calendar(to="2026-07-26", page=0, include_report_times=False)

    assert fixture_server.requests[0].target == "/earnings-calendar?to=2026-07-26&page=0&includeReportTimes=false"
    assert len(rows) == 1


def test_stock_splits_with_zero_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``stock_splits`` maps to ``/splits`` and sends ``limit=0`` when asked."""
    fixture_server.route("/splits", load_fixture("stock_splits.json"))
    rows = client.calendar.stock_splits("AAPL", limit=0)

    assert fixture_server.requests[0].target == "/splits?symbol=AAPL&limit=0"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockSplitEvent)
    assert row.symbol == "AAPL"
    assert row.date == datetime.date(2020, 8, 31)
    assert (row.numerator, row.denominator) == (4, 1)
    assert row.split_type == "stock-split"


def test_stock_splits_decode_fractional_terms(client: Any, fixture_server: FixtureServer) -> None:
    """Issue #339: a fractional or ``1.0``-form split term decodes as a ``float``."""
    fixture_server.route("/splits", load_fixture("stock_splits_fractional_synthetic.json"))
    rows = client.calendar.stock_splits("AAPL")

    assert len(rows) == 1
    assert isinstance(rows[0].numerator, float)
    assert (rows[0].numerator, rows[0].denominator) == (1.5, 1.0)


def test_stock_splits_without_options_sends_only_the_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``stock_splits`` omits ``limit`` when it is not given."""
    fixture_server.route("/splits", load_fixture("stock_splits.json"))
    rows = client.calendar.stock_splits("AAPL")

    assert fixture_server.requests[0].target == "/splits?symbol=AAPL"
    assert len(rows) == 1


def test_stock_splits_calendar_with_every_option(client: Any, fixture_server: FixtureServer) -> None:
    """``stock_splits_calendar`` maps to ``/splits-calendar`` with ``from``, ``to``, ``page``."""
    fixture_server.route("/splits-calendar", load_fixture("stock_splits_calendar.json"))
    rows = client.calendar.stock_splits_calendar(from_="2026-01-27", to=TO_2026_04_27, page=0)

    assert fixture_server.requests[0].target == "/splits-calendar?from=2026-01-27&to=2026-04-27&page=0"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, StockSplitEvent)
    assert row.symbol == "WHLR"
    assert row.date == datetime.date(2026, 7, 28)
    assert (row.numerator, row.denominator) == (1, 5)


def test_stock_splits_calendar_omits_from_independently(client: Any, fixture_server: FixtureServer) -> None:
    """``stock_splits_calendar`` sends ``to`` and ``page`` without ``from``."""
    fixture_server.route("/splits-calendar", load_fixture("stock_splits_calendar.json"))
    rows = client.calendar.stock_splits_calendar(to="2026-04-27", page=0)

    assert fixture_server.requests[0].target == "/splits-calendar?to=2026-04-27&page=0"
    assert len(rows) == 1


def test_earnings_calendar_decodes_fractional_revenue(client: Any, fixture_server: FixtureServer) -> None:
    """Issue #340: fractional and integral-float revenue decodes as ``float``."""
    fixture_server.route("/earnings-calendar", load_fixture("earnings_calendar_fractional_synthetic.json"))
    rows = client.calendar.earnings_calendar()

    assert len(rows) == 1
    assert isinstance(rows[0].revenue_estimated, float)
    assert rows[0].revenue_estimated == 1_086_300_000.5
    assert rows[0].revenue_actual == 1_101_500_000.0
