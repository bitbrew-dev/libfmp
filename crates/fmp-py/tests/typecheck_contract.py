"""Static-only contract exercised by pyright and mypy against the shipped stubs."""

import datetime

import fmp
from fmp import BinaryPayload, FmpClient, FmpError
from fmp.bulk import BulkNamespace
from fmp.bulk.eod import BulkEodBar
from fmp.bulk.income import BulkIncomeStatement
from fmp.bulk.metrics import BulkEarningsSurprise
from fmp.calendar import CalendarNamespace, DividendEvent, EarningsEvent, IpoCalendarEvent
from fmp.company import CompanyProfile
from fmp.errors import FmpStatusError, FmpValidationError
from fmp.quote import AftermarketTrade, Quote, QuoteShort, StockPriceChange
from fmp.screener import CompanyScreenerEntry, ScreenerNamespace
from fmp.statements import StatementsNamespace
from fmp.statements.income import IncomeStatement, StatementsIncomeNamespace
from fmp.statements.reports import FinancialReportDate
from fmp.statements.summaries import LatestFinancialStatement


def check_statements_contract(client: FmpClient) -> None:
    """Type-check the two-level nested namespace and its bespoke result types."""
    statements: StatementsNamespace = client.statements
    income: StatementsIncomeNamespace = statements.income
    rows: list[IncomeStatement] = income.statement("AAPL", period="annual", limit=5)
    reported_on: datetime.date = rows[0].date
    revenue: int = rows[0].revenue
    dates: list[FinancialReportDate] = client.statements.reports.dates("AAPL")
    link: str = dates[0].expose_secret_url_json()
    workbook: BinaryPayload = client.statements.reports.xlsx("AAPL", 2024, "FY")
    latest: list[LatestFinancialStatement] = client.statements.summaries.latest_financial_statements(page=0, limit=10)
    added: datetime.datetime = latest[0].date_added
    _ = (reported_on, revenue, link, workbook.data, added)


def check_calendar_contract(client: FmpClient) -> None:
    """Type-check the flat calendar namespace, its date keywords, and the renamed fields."""
    calendar: CalendarNamespace = client.calendar
    dividends: list[DividendEvent] = calendar.dividends("AAPL", limit=5)
    yield_: float = dividends[0].yield_
    declared: datetime.date | None = dividends[0].declaration_date
    earnings: list[EarningsEvent] = calendar.earnings_calendar(
        from_=datetime.date(2026, 4, 27), to="2026-07-26", page=0, include_report_times=True
    )
    eps_actual: float | None = earnings[0].eps_actual
    ipos: list[IpoCalendarEvent] = calendar.ipos_calendar(from_="2026-03-06", to=datetime.date(2026, 6, 6))
    ipo_date: datetime.date = ipos[0].date
    _ = (yield_, declared, eps_actual, ipo_date)


def check_bulk_contract(client: FmpClient) -> None:
    """Type-check the constructor-only bulk arguments and the cross-domain profile rows."""
    bulk: BulkNamespace = client.bulk
    profiles: list[CompanyProfile] = bulk.company_profiles("0")
    ipo_date: datetime.date = profiles[0].ipo_date
    statements: list[BulkIncomeStatement] = client.bulk.income_statements(2026, "Q1")
    accepted: datetime.datetime = statements[0].accepted_date
    revenue: str = statements[0].revenue
    surprises: list[BulkEarningsSurprise] = client.bulk.earnings_surprises(year=2026)
    eps_actual: str = surprises[0].eps_actual
    bars: list[BulkEodBar] = client.bulk.eod(datetime.date(2024, 10, 22))
    more_bars: list[BulkEodBar] = client.bulk.eod(date="2024-10-22")
    close: str = bars[0].close
    _ = (ipo_date, accepted, revenue, eps_actual, close, more_bars)


def check_screener_contract(client: FmpClient) -> None:
    """Type-check the keyword-only filter surface and the screener row type."""
    screener: ScreenerNamespace = client.screener
    everything: list[CompanyScreenerEntry] = screener.companies()
    filtered: list[CompanyScreenerEntry] = client.screener.companies(
        market_cap_more_than=1_000_000_000,
        sector="Technology",
        beta_lower_than=1.5,
        is_etf=False,
        limit=100,
    )
    market_cap: int = filtered[0].market_cap
    beta: float = everything[0].beta
    is_fund: bool = filtered[0].is_fund
    _ = (market_cap, beta, is_fund)


def check_public_contract(client: FmpClient) -> None:
    """Type-check the namespaced call shapes and the model fields."""
    short_rows: list[QuoteShort] = client.quote.short("AAPL")
    full_rows: list[Quote] = client.quote.full("AAPL")
    fund_rows: list[QuoteShort] = client.quote.mutual_funds()
    batch_rows: list[Quote] = client.quote.batch_quote(["AAPL", "MSFT"])
    trades: list[AftermarketTrade] = client.quote.batch_aftermarket_trade("AAPL")
    changes: list[StockPriceChange] = client.quote.stock_price_change("AAPL")
    exchange_rows: list[QuoteShort] = client.quote.exchange("NASDAQ")
    trade_size: int = trades[0].trade_size
    ten_years: float = changes[0].ten_years
    symbol: str = short_rows[0].symbol
    price: float = full_rows[0].price
    volume: int = fund_rows[0].volume
    market_cap: int | None = full_rows[0].market_cap
    exports: list[str] = fmp.__all__
    version: str = fmp.__version__
    _ = (symbol, price, volume, market_cap, exports, version)
    _ = (batch_rows, exchange_rows, trade_size, ten_years)


def check_binary_contract() -> None:
    """Type-check the binary payload fields."""
    payload: BinaryPayload = BinaryPayload(b"", "application/octet-stream")
    data: bytes = payload.data
    content_type: str = payload.content_type
    disposition: str | None = payload.content_disposition
    byte_len: int = payload.byte_len
    _ = (data, content_type, disposition, byte_len)


def check_error_contract(client: FmpClient) -> None:
    """Type-check the exception hierarchy and its structured attributes."""
    try:
        client.quote.short("AAPL")
    except FmpStatusError as error:
        status: int | None = error.status
        endpoint: str | None = error.endpoint
        body: str | None = error.body
        body_truncated: bool | None = error.body_truncated
        _ = (status, endpoint, body, body_truncated)
    except FmpValidationError as error:
        category: str | None = error.category
        _ = category
    except FmpError as error:
        base: FmpError = error
        top_level: fmp.FmpError = error
        _ = (base, top_level)


FmpClient()
FmpClient(
    token="proxy-token",
    base_url="https://proxy.example",
    auth_mode="custom_header",
    auth_name="X-Proxy-Token",
    headers={"X-Tenant": "blue"},
    timeout=5.0,
    connect_timeout=2.0,
    max_response_body_bytes=67_108_864,
    danger_allow_insecure_authentication=False,
    follow_redirects=False,
)
