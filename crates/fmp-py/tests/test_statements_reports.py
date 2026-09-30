"""Runtime contract of ``client.statements.reports`` and ``client.statements.summaries``.

``reports`` closes the two bespoke paths end to end: ``xlsx`` returns a
``BinaryPayload`` (#174) and ``dates`` returns rows whose download links are
redacted ``SecretUrl`` fields (#175). ``summaries`` covers the ``page``/``limit``
shape without a symbol. Targets come from ``statements_report_endpoints.rs``
and ``statements_summary_endpoints.rs``.
"""

import datetime
from collections.abc import Iterator
from types import SimpleNamespace
from typing import Any

import pytest
from conftest import FixtureServer, load_fixture
from fmp import BinaryPayload
from fmp.statements.reports import FinancialReportDate, FinancialReportJson, StatementsReportsNamespace
from fmp.statements.summaries import (
    EnterpriseValue,
    FinancialScore,
    LatestFinancialStatement,
    OwnerEarnings,
    StatementsSummariesNamespace,
)

XLSX = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
WORKBOOK = b"PK\x03\x04\xff\xfe\x80\x00 not utf-8"
REDACTED = "[REDACTED URL]"


@pytest.fixture
def reveal_off() -> Iterator[None]:
    """Pin the process-wide reveal flag off and restore it afterwards."""
    from fmp import reveal_secret_urls, set_reveal_secret_urls

    previous = reveal_secret_urls()
    set_reveal_secret_urls(False)
    try:
        yield
    finally:
        set_reveal_secret_urls(previous)


def test_reports_and_summaries_namespaces(client: Any) -> None:
    """Both sub-namespaces are the generated classes."""
    assert isinstance(client.statements.reports, StatementsReportsNamespace)
    assert isinstance(client.statements.summaries, StatementsSummariesNamespace)


def test_report_dates_redact_links_until_exposed(client: Any, fixture_server: FixtureServer, reveal_off: None) -> None:
    """``reports.dates`` rows hide the keyed links behind ``expose_secret_url_*``."""
    body = load_fixture("financial_reports_dates.json")
    fixture_server.route("/financial-reports-dates", body)
    rows = client.statements.reports.dates("AAPL")

    assert fixture_server.requests[0].target == "/financial-reports-dates?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, FinancialReportDate)
    assert (row.symbol, row.fiscal_year, row.period) == ("AAPL", 2026, "Q2")
    assert not hasattr(row, "link_json")
    assert repr(row) == (
        f"FinancialReportDate(symbol='AAPL', fiscal_year=2026, period='Q2', link_json={REDACTED}, link_xlsx={REDACTED})"
    )
    assert row.expose_secret_url_json() == body[0]["linkJson"]
    assert row.expose_secret_url_xlsx() == body[0]["linkXlsx"]


def test_report_json_with_year_and_fiscal_period(client: Any, fixture_server: FixtureServer) -> None:
    """``reports.json`` takes ``symbol``, ``year``, ``period`` positionally and returns one report."""
    fixture_server.route("/financial-reports-json", load_fixture("financial_reports_json.json"))
    report = client.statements.reports.json("AAPL", 2023, "Q1")

    assert fixture_server.requests[0].target == "/financial-reports-json?symbol=AAPL&year=2023&period=Q1"
    assert isinstance(report, FinancialReportJson)
    assert (report.symbol, report.period, report.year) == ("AAPL", "Q1", "2023")
    assert isinstance(report.sections, dict)
    assert len(report.sections) == 3
    assert report.sections["Revenue - Additional Informatio"][2] == {"Total deferred revenue": [12.6, 12.4]}


def test_report_json_rejects_a_list_body(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """A bare array is not a report: the single-object decode fails instead of taking the first row."""
    fixture_server.route("/financial-reports-json", [load_fixture("financial_reports_json.json")])
    with pytest.raises(errors.FmpDecodeError):
        client.statements.reports.json("AAPL", 2023, "Q1")


def test_report_xlsx_returns_a_binary_payload(client: Any, fixture_server: FixtureServer) -> None:
    """``reports.xlsx`` returns the workbook bytes plus the response media type."""
    fixture_server.route("/financial-reports-xlsx", WORKBOOK, content_type=XLSX)
    payload = client.statements.reports.xlsx("AAPL", 2022, "FY")

    assert fixture_server.requests[0].target == "/financial-reports-xlsx?symbol=AAPL&year=2022&period=FY"
    assert isinstance(payload, BinaryPayload)
    assert type(payload.data) is bytes
    assert payload.data == WORKBOOK
    assert payload.content_type == XLSX
    assert payload.byte_len == len(WORKBOOK)
    assert len(payload) == len(WORKBOOK)


def test_latest_financial_statements_with_page_and_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``summaries.latest_financial_statements`` is keyword-only: ``page`` then ``limit``."""
    fixture_server.route("/latest-financial-statements", load_fixture("latest_financial_statements.json"))
    rows = client.statements.summaries.latest_financial_statements(page=0, limit=250)

    assert fixture_server.requests[0].target == "/latest-financial-statements?page=0&limit=250"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, LatestFinancialStatement)
    assert (row.symbol, row.calendar_year, row.period) == ("UFPI", 2026, "Q2")
    assert row.date == datetime.date(2026, 6, 27)
    assert row.date_added == datetime.datetime(2026, 7, 30, 13, 17, 26)
    with pytest.raises(TypeError):
        client.statements.summaries.latest_financial_statements(0)


def test_null_altman_z_score_decodes_as_none(client: Any, fixture_server: FixtureServer) -> None:
    """Issue #368: a null ``altmanZScore`` decodes as ``None``."""
    scores = load_fixture("financial_scores.json")
    scores[0]["altmanZScore"] = None
    fixture_server.route("/financial-scores", scores)

    row = client.statements.summaries.financial_scores("AAPL")[0]
    assert row.altman_z_score is None
    assert row.piotroski_score == 9


def test_financial_scores_takes_only_a_symbol(client: Any, fixture_server: FixtureServer) -> None:
    """``summaries.financial_scores`` sends the symbol alone."""
    fixture_server.route("/financial-scores", load_fixture("financial_scores.json"))
    rows = client.statements.summaries.financial_scores("AAPL")

    assert fixture_server.requests[0].target == "/financial-scores?symbol=AAPL"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, FinancialScore)
    assert row.symbol == "AAPL"
    assert row.altman_z_score == pytest.approx(14.041374927993303, rel=1e-15)
    assert row.piotroski_score == 9
    assert row.market_cap == 5_042_169_135_511


def test_owner_earnings_with_limit(client: Any, fixture_server: FixtureServer) -> None:
    """``summaries.owner_earnings`` takes ``limit`` and no period selector."""
    fixture_server.route("/owner-earnings", load_fixture("owner_earnings.json"))
    rows = client.statements.summaries.owner_earnings("AAPL", limit=5)

    assert fixture_server.requests[0].target == "/owner-earnings?symbol=AAPL&limit=5"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, OwnerEarnings)
    assert row.date == datetime.date(2026, 3, 28)
    assert row.period == "Q2"
    assert row.owners_earnings == 28_861_994_500
    assert row.maintenance_capex == 159_994_500
    with pytest.raises(TypeError):
        client.statements.summaries.owner_earnings("AAPL", period="FY")


def test_enterprise_values_with_full_year_period(client: Any, fixture_server: FixtureServer) -> None:
    """``summaries.enterprise_values`` accepts the ``FY`` statement period."""
    fixture_server.route("/enterprise-values", load_fixture("enterprise_values.json"))
    rows = client.statements.summaries.enterprise_values("AAPL", period="FY")

    assert fixture_server.requests[0].target == "/enterprise-values?symbol=AAPL&period=FY"
    assert len(rows) == 1
    row = rows[0]
    assert isinstance(row, EnterpriseValue)
    assert row.date == datetime.date(2025, 9, 27)
    assert row.stock_price == 255.46
    assert row.enterprise_value == 3_895_186_810_000
    assert row.number_of_shares == 14_948_500_000


def test_retrieval_frequency_is_not_a_fiscal_period(
    client: Any, fixture_server: FixtureServer, errors: SimpleNamespace
) -> None:
    """Report queries take ``Q1``..``FY`` only; ``annual`` is rejected locally."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.statements.reports.json("AAPL", 2022, "annual")
    assert str(raised.value) == "period: fiscal period must be one of Q1, Q2, Q3, Q4, FY"
    assert fixture_server.requests == []


def test_negative_page_names_the_argument(client: Any, fixture_server: FixtureServer, errors: SimpleNamespace) -> None:
    """A negative ``page`` fails before any request with the keyword as the prefix."""
    with pytest.raises(errors.FmpValidationError) as raised:
        client.statements.summaries.latest_financial_statements(page=-1)
    assert str(raised.value).startswith("page: ")
    assert fixture_server.requests == []
