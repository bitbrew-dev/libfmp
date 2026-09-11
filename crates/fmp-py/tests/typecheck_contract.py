"""Static-only contract exercised by pyright and mypy against the shipped stubs."""

import datetime
from typing import Any

import fmp
from fmp import BinaryPayload, FmpClient, FmpError
from fmp.bulk import BulkNamespace
from fmp.bulk.eod import BulkEodBar
from fmp.bulk.income import BulkIncomeStatement
from fmp.bulk.metrics import BulkEarningsSurprise
from fmp.calendar import CalendarNamespace, DividendEvent, EarningsEvent, IpoCalendarEvent
from fmp.chart import (
    ChartNamespace,
    StockChartAdjustedBar,
    StockChartFullBar,
    StockChartIntradayBar,
    StockChartLightBar,
)
from fmp.commodities import CommoditiesNamespace, CommodityListing
from fmp.company import CompanyNamespace, CompanyProfile, ExecutiveCompensationBenchmark, MarketCapitalizationRecord
from fmp.congressional import (
    CongressionalMemberNetWorthAggregate,
    CongressionalMemberNetWorthEntry,
    CongressionalMemberProfile,
    CongressionalNamespace,
    CongressionalNetWorthRange,
    CongressionalTrade,
)
from fmp.directory import AvailableExchange, CikEntry, DirectoryNamespace, EarningsTranscriptAvailability, SymbolChange
from fmp.errors import FmpStatusError, FmpValidationError
from fmp.funds import EtfFundHolding, EtfFundInfo, FundDisclosure, FundsNamespace
from fmp.indexes import IndexConstituent, IndexesNamespace, IndexListing
from fmp.institutional_ownership import (
    Form13fFilingDate,
    HolderPerformanceSummary,
    InstitutionalHolderAnalytics,
    InstitutionalOwnershipFiling,
    InstitutionalOwnershipNamespace,
)
from fmp.market import IndustryPe, MarketMover, MarketNamespace, SectorPerformance
from fmp.news import FmpArticle, NewsArticle, NewsNamespace
from fmp.quote import AftermarketTrade, Quote, QuoteShort, StockPriceChange
from fmp.screener import CompanyScreenerEntry, ScreenerNamespace
from fmp.sec_filings import SecCompanyProfile, SecCompanySearchResult, SecFiling, SecFilingsNamespace
from fmp.statements import StatementsNamespace
from fmp.statements.income import IncomeStatement, StatementsIncomeNamespace
from fmp.statements.reports import FinancialReportDate
from fmp.statements.summaries import LatestFinancialStatement
from fmp.technical_indicators import (
    RelativeStrengthIndexBar,
    SimpleMovingAverageBar,
    TechnicalIndicatorsNamespace,
    WilliamsBar,
)


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


def check_company_contract(client: FmpClient) -> None:
    """Type-check the flat company namespace, its date keywords, and the batch list input."""
    company: CompanyNamespace = client.company
    profiles: list[CompanyProfile] = company.profile("AAPL")
    ipo_date: datetime.date = profiles[0].ipo_date
    history: list[MarketCapitalizationRecord] = client.company.historical_market_capitalization(
        "AAPL", limit=5001, from_=datetime.date(2026, 4, 16), to="2026-07-16"
    )
    market_cap: int = history[0].market_cap
    batch: list[MarketCapitalizationRecord] = client.company.market_capitalization_batch(["AAPL", "MSFT"])
    benchmarks: list[ExecutiveCompensationBenchmark] = client.company.executive_compensation_benchmark(year="2024")
    average: float = benchmarks[0].average_compensation
    _ = (ipo_date, market_cap, batch, average)


def check_directory_contract(client: FmpClient) -> None:
    """Type-check the keyword-only directory queries, the bool-typed flags, and the date row."""
    directory: DirectoryNamespace = client.directory
    changes: list[SymbolChange] = directory.symbol_changes(invalid=False, limit=100)
    changed_on: datetime.date = changes[0].date
    exchanges: list[AvailableExchange] = client.directory.available_exchanges(extended=True)
    suffix: str = exchanges[0].symbol_suffix
    entries: list[CikEntry] = client.directory.cik_list(page=0, limit=1000)
    cik: str = entries[0].cik
    transcripts: list[EarningsTranscriptAvailability] = client.directory.earnings_transcript_list()
    count: str = transcripts[0].no_of_transcripts
    _ = (changed_on, suffix, cik, count)


def check_congressional_contract(client: FmpClient) -> None:
    """Type-check the keyword-only filters, the required member ID, and the nested net-worth rows."""
    congressional: CongressionalNamespace = client.congressional
    trades: list[CongressionalTrade] = congressional.senate_trades("AAPL", page=0, limit=250)
    disclosed: datetime.date = trades[0].disclosure_date
    gains: str | None = trades[0].capital_gains_over_200_usd
    profiles: list[CongressionalMemberProfile] = client.congressional.profiles(
        active=True, member_id="P000197", latest_party="Democrat", latest_position="Senator", page=0, limit=20
    )
    born: datetime.date = profiles[0].birth_date
    years_active: float = profiles[0].years_active
    entries: list[CongressionalMemberNetWorthEntry] = client.congressional.net_worth("P000197", limit=100)
    value_range: CongressionalNetWorthRange | None = entries[0].value_range
    totals: list[CongressionalMemberNetWorthAggregate] = client.congressional.net_worth_aggregated(
        "P000197", totals_col="stock"
    )
    total: int = totals[0].total
    _ = (disclosed, gains, born, years_active, value_range, total)


def check_funds_contract(client: FmpClient) -> None:
    """Type-check the funds namespace, the required period ints, the keyword-only CIK, and the aliased date row."""
    funds: FundsNamespace = client.funds
    holdings: list[EtfFundHolding] = funds.etf_holdings("SPY")
    updated: datetime.datetime = holdings[0].updated_at
    info: list[EtfFundInfo] = client.funds.etf_info("SPY")
    inception: datetime.date = info[0].inception_date
    exposure: float = info[0].sectors_list[0].exposure
    positions: list[FundDisclosure] = client.funds.fund_disclosures("VWO", 2023, 4, cik="0000857489")
    accepted: datetime.datetime = positions[0].accepted_date
    dates: list[Form13fFilingDate] = client.funds.fund_disclosure_dates("VWO", cik="0000036405")
    quarter: int = dates[0].quarter
    _ = (updated, inception, exposure, accepted, quarter)


def check_indexes_contract(client: FmpClient) -> None:
    """Type-check the indexes namespace, its keyword-only date filters, and the cross-module rows."""
    indexes: IndexesNamespace = client.indexes
    listings: list[IndexListing] = indexes.list()
    currency: str = listings[0].currency
    bars: list[StockChartFullBar] = indexes.chart_full("^VIX", from_=datetime.date(2026, 1, 27), to="2026-04-27")
    bar_date: datetime.date = bars[0].date
    vwap: float = bars[0].vwap
    intraday: list[StockChartIntradayBar] = client.indexes.chart_one_minute("^VIX", to=datetime.date(2024, 3, 1))
    bar_time: datetime.datetime = intraday[0].date
    members: list[IndexConstituent] = client.indexes.sp500_constituents()
    first_added: datetime.date | None = members[0].date_first_added
    founded: datetime.date = members[0].founded
    _ = (currency, bar_date, vwap, bar_time, first_added, founded)


def check_institutional_ownership_contract(client: FmpClient) -> None:
    """Type-check the flat institutional-ownership namespace, its period arguments, and the datetime rows."""
    institutional_ownership: InstitutionalOwnershipNamespace = client.institutional_ownership
    filings: list[InstitutionalOwnershipFiling] = institutional_ownership.latest_filings(page=0, limit=100)
    accepted: datetime.datetime = filings[0].accepted_date
    reported_on: datetime.date = filings[0].date
    analytics: list[InstitutionalHolderAnalytics] = client.institutional_ownership.holder_analytics(
        "AAPL", 2023, 3, page=0, limit=10
    )
    first_added: datetime.date = analytics[0].first_added
    is_new: bool = analytics[0].is_new
    summaries: list[HolderPerformanceSummary] = client.institutional_ownership.holder_performance_summary(
        "0001067983", page=0
    )
    turnover: float = summaries[0].turnover
    _ = (accepted, reported_on, first_added, is_new, turnover)


def check_chart_contract(client: FmpClient) -> None:
    """Type-check the flat chart namespace, its keyword-only dates and flags, and the three row shapes."""
    chart: ChartNamespace = client.chart
    light: list[StockChartLightBar] = chart.light("AAPL", from_=datetime.date(2026, 4, 30), to="2026-07-30")
    traded_on: datetime.date = light[0].date
    price: float = light[0].price
    adjusted: list[StockChartAdjustedBar] = client.chart.dividend_adjusted("AAPL", to=datetime.date(2026, 7, 30))
    adj_close: float = adjusted[0].adj_close
    intraday: list[StockChartIntradayBar] = client.chart.one_minute(
        "AAPL", from_="2024-01-01", to=datetime.date(2024, 3, 1), nonadjusted=False, extended=True
    )
    bar_time: datetime.datetime = intraday[0].date
    volume: int = intraday[0].volume
    _ = (traded_on, price, adj_close, bar_time, volume)


def check_market_contract(client: FmpClient) -> None:
    """Type-check the market namespace, its required date or sector, the keyword-only filters, and the mover rows."""
    market: MarketNamespace = client.market
    snapshot: list[SectorPerformance] = market.sector_performance_snapshot(
        datetime.date(2024, 2, 1), exchange="NASDAQ", sector="Energy"
    )
    snapshot_date: datetime.date = snapshot[0].date
    average_change: float = snapshot[0].average_change
    history: list[IndustryPe] = client.market.historical_industry_pe(
        "Biotechnology", exchange="NASDAQ", from_=datetime.date(2024, 2, 1), to="2024-03-01"
    )
    pe: float = history[0].pe
    movers: list[MarketMover] = client.market.biggest_gainers()
    changes_percentage: float = movers[0].changes_percentage
    _ = (snapshot_date, average_change, pe, changes_percentage)


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


def check_sec_filings_contract(client: FmpClient) -> None:
    """Type-check the date-keyed filing feeds, the lookups, and the profile row."""
    sec_filings: SecFilingsNamespace = client.sec_filings
    filings: list[SecFiling] = sec_filings.by_form_type(
        "8-K", datetime.date(2024, 1, 1), "2024-03-01", page=0, limit=100
    )
    accepted: datetime.datetime = filings[0].accepted_date
    has_financials: bool | None = filings[0].has_financials
    companies: list[SecCompanySearchResult] = client.sec_filings.search_companies_by_cik("0000320193")
    cik: str = companies[0].cik
    profiles: list[SecCompanyProfile] = client.sec_filings.company_profile("AAPL", cik_a="0000320193")
    ipo_date: datetime.date = profiles[0].ipo_date
    raw: list[dict[str, Any]] = client.sec_filings.search_industry_classifications(
        symbol="AAPL", cik="0000320193", sic_code="3571"
    )
    _ = (accepted, has_financials, cik, ipo_date, raw)


def check_news_contract(client: FmpClient) -> None:
    """Type-check the keyword-only news feeds, the ticker-list searches, and the two row types."""
    news: NewsNamespace = client.news
    articles: list[FmpArticle] = news.fmp_articles(page=0, limit=20)
    authored_on: datetime.datetime = articles[0].date
    tickers: str = articles[0].tickers
    general: list[NewsArticle] = client.news.latest_general_news(
        from_=datetime.date(2026, 1, 27), to="2026-04-28", page=100, limit=251
    )
    symbol: str | None = general[0].symbol
    published: datetime.datetime = general[0].published_date
    searched: list[NewsArticle] = client.news.search_stock_news(["AAPL", "MSFT"], from_="2026-01-27")
    single: list[NewsArticle] = client.news.search_forex_news("EURUSD", limit=0)
    publisher: str = searched[0].publisher
    _ = (authored_on, tickers, symbol, published, single, publisher)


def check_technical_indicators_contract(client: FmpClient) -> None:
    """Type-check the shared indicator signature, its keyword-only dates, and the bar row fields."""
    technical_indicators: TechnicalIndicatorsNamespace = client.technical_indicators
    sma: list[SimpleMovingAverageBar] = technical_indicators.simple_moving_average(
        "AAPL", 10, "1day", from_=datetime.date(2026, 3, 1), to="2026-06-01"
    )
    bar_time: datetime.datetime = sma[0].date
    volume: int = sma[0].volume
    average: float = sma[0].sma
    rsi: list[RelativeStrengthIndexBar] = client.technical_indicators.relative_strength_index("AAPL", 14, "1hour")
    strength: float = rsi[0].rsi
    williams: list[WilliamsBar] = client.technical_indicators.williams(
        "AAPL", 14, "15min", to=datetime.date(2026, 6, 1)
    )
    oscillator: float = williams[0].williams
    _ = (bar_time, volume, average, strength, oscillator)


def check_commodities_contract(client: FmpClient) -> None:
    """Type-check the commodities namespace, its keyword-only date filters, and the cross-module rows."""
    commodities: CommoditiesNamespace = client.commodities
    listings: list[CommodityListing] = commodities.list()
    exchange: str | None = listings[0].exchange
    trade_month: str = listings[0].trade_month
    quotes: list[Quote] = client.commodities.quote("GCUSD")
    market_cap: int | None = quotes[0].market_cap
    bars: list[StockChartFullBar] = commodities.chart_full("GCUSD", from_=datetime.date(2026, 1, 27), to="2026-04-27")
    bar_date: datetime.date = bars[0].date
    vwap: float = bars[0].vwap
    intraday: list[StockChartIntradayBar] = client.commodities.chart_one_hour("GCUSD", to=datetime.date(2024, 3, 1))
    bar_time: datetime.datetime = intraday[0].date
    _ = (exchange, trade_month, market_cap, bar_date, vwap, bar_time)


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
