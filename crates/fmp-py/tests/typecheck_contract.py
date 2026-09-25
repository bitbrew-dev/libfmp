"""Static-only contract exercised by pyright and mypy against the shipped stubs."""

import datetime
from typing import Any

import fmp
from fmp import BinaryPayload, FmpClient, FmpError
from fmp.analyst import AnalystNamespace, FinancialEstimate, PriceTargetConsensus, StockGrade
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
from fmp.commitment_of_traders import (
    CommitmentOfTradersNamespace,
    CotAnalysis,
    CotReport,
    CotReportListing,
)
from fmp.commodities import CommoditiesNamespace, CommodityListing
from fmp.company import CompanyMarketCapitalization, CompanyNamespace, CompanyProfile, ExecutiveCompensationBenchmark
from fmp.congressional import (
    CongressionalMemberNetWorthAggregate,
    CongressionalMemberNetWorthEntry,
    CongressionalMemberProfile,
    CongressionalNamespace,
    CongressionalNetWorthRange,
    CongressionalTrade,
)
from fmp.crypto import CryptocurrencyListing, CryptoNamespace
from fmp.dcf import CustomDcfValuation, CustomLeveredDcfValuation, DcfNamespace, DcfValuation
from fmp.directory import AvailableExchange, CikEntry, DirectoryNamespace, EarningsTranscriptAvailability, SymbolChange
from fmp.economics import (
    EconomicCalendarEvent,
    EconomicIndicatorObservation,
    EconomicsNamespace,
    MarketRiskPremium,
    TreasuryRate,
)
from fmp.errors import FmpStatusError, FmpValidationError
from fmp.esg import EsgBenchmark, EsgDisclosure, EsgNamespace, EsgRating
from fmp.forex import ForexNamespace, ForexPair
from fmp.fundraising import (
    CrowdfundingOffering,
    CrowdfundingOfferingSearchResult,
    FundraisingNamespace,
    RegulationDOffering,
    RegulationDOfferingSearchResult,
)
from fmp.funds import EtfFundHolding, EtfFundInfo, FundDisclosure, FundsNamespace
from fmp.indexes import IndexConstituent, IndexesNamespace, IndexListing
from fmp.insider_trading import (
    BeneficialOwnershipAcquisition,
    InsiderTrade,
    InsiderTradeStatistics,
    InsiderTradingNamespace,
    InsiderTransactionType,
)
from fmp.institutional_ownership import (
    Form13fFilingDate,
    HolderPerformanceSummary,
    InstitutionalHolderAnalytics,
    InstitutionalOwnershipFiling,
    InstitutionalOwnershipNamespace,
)
from fmp.market import IndustryPe, MarketMover, MarketNamespace, SectorPerformance
from fmp.market_hours import ExchangeHoliday, ExchangeMarketHours, MarketHoursNamespace
from fmp.news import FmpArticle, NewsArticle, NewsNamespace
from fmp.quote import AftermarketTrade, Quote, QuoteShort, StockPriceChange
from fmp.screener import CompanyScreenerEntry, ScreenerNamespace
from fmp.search import CikSearchResult, ExchangeVariant, SearchNamespace, SymbolSearchResult
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
from fmp.tipranks import (
    TipRanksAnalystProfile,
    TipRanksAnalystSummary,
    TipranksNamespace,
    TipRanksPointInTimeRating,
    TipRanksRatingSearchResult,
    TipRanksRecommendationCounts,
)
from fmp.transcripts import (
    EarningsTranscript,
    EarningsTranscriptDate,
    LatestEarningsTranscript,
    TranscriptsNamespace,
)


def check_statements_contract(client: FmpClient) -> None:
    """Type-check the two-level nested namespace and its bespoke result types."""
    statements: StatementsNamespace = client.statements
    income: StatementsIncomeNamespace = statements.income
    rows: list[IncomeStatement] = income.statement("AAPL", period="annual", limit=5)
    reported_on: datetime.date = rows[0].date
    revenue: float = rows[0].revenue
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
    history: list[CompanyMarketCapitalization] = client.company.historical_market_capitalization(
        "AAPL", limit=5001, from_=datetime.date(2026, 4, 16), to="2026-07-16"
    )
    market_cap: float = history[0].market_cap
    batch: list[CompanyMarketCapitalization] = client.company.batch_market_capitalization(["AAPL", "MSFT"])
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
    positions: list[FundDisclosure] = client.funds.disclosures("VWO", 2023, 4, cik="0000857489")
    accepted: datetime.datetime = positions[0].accepted_date
    dates: list[Form13fFilingDate] = client.funds.disclosure_dates("VWO", cik="0000036405")
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


def check_forex_contract(client: FmpClient) -> None:
    """Type-check the flat forex namespace, its keyword-only date filters, and the cross-module rows."""
    forex: ForexNamespace = client.forex
    pairs: list[ForexPair] = forex.list()
    from_currency: str = pairs[0].from_currency
    quotes: list[Quote] = client.forex.quote("EURUSD")
    market_cap: float | None = quotes[0].market_cap
    compact: list[QuoteShort] = client.forex.quote_short("EURUSD")
    price: float = compact[0].price
    bars: list[StockChartFullBar] = forex.chart_full("EURUSD", from_=datetime.date(2026, 1, 27), to="2026-04-27")
    bar_date: datetime.date = bars[0].date
    vwap: float = bars[0].vwap
    light: list[StockChartLightBar] = client.forex.chart_light("EURUSD", from_="2026-01-27")
    intraday: list[StockChartIntradayBar] = client.forex.chart_one_hour("EURUSD", to=datetime.date(2024, 3, 1))
    bar_time: datetime.datetime = intraday[0].date
    _ = (from_currency, market_cap, price, bar_date, vwap, light, bar_time)


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
    volume: float = intraday[0].volume
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
    market_cap: float = filtered[0].market_cap
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
    volume: float = sma[0].volume
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
    market_cap: float | None = quotes[0].market_cap
    bars: list[StockChartFullBar] = commodities.chart_full("GCUSD", from_=datetime.date(2026, 1, 27), to="2026-04-27")
    bar_date: datetime.date = bars[0].date
    vwap: float = bars[0].vwap
    intraday: list[StockChartIntradayBar] = client.commodities.chart_one_hour("GCUSD", to=datetime.date(2024, 3, 1))
    bar_time: datetime.datetime = intraday[0].date
    _ = (exchange, trade_month, market_cap, bar_date, vwap, bar_time)


def check_crypto_contract(client: FmpClient) -> None:
    """Type-check the crypto namespace, its keyword-only date filters, and the cross-module rows."""
    crypto: CryptoNamespace = client.crypto
    listings: list[CryptocurrencyListing] = crypto.list()
    ico_date: datetime.date = listings[0].ico_date
    total_supply: float = listings[0].total_supply
    quotes: list[Quote] = client.crypto.quote("BTCUSD")
    market_cap: float | None = quotes[0].market_cap
    bars: list[StockChartFullBar] = client.crypto.chart_full(
        "BTCUSD", from_=datetime.date(2026, 1, 27), to="2026-04-27"
    )
    bar_date: datetime.date = bars[0].date
    intraday: list[StockChartIntradayBar] = client.crypto.chart_one_hour("BTCUSD", to=datetime.date(2024, 3, 1))
    bar_time: datetime.datetime = intraday[0].date
    _ = (ico_date, total_supply, market_cap, bar_date, bar_time)


def check_analyst_contract(client: FmpClient) -> None:
    """Type-check the flat analyst namespace, its required period, the keyword-only paging, and the date rows."""
    analyst: AnalystNamespace = client.analyst
    estimates: list[FinancialEstimate] = analyst.financial_estimates("AAPL", "quarter", page=0, limit=10)
    estimated_on: datetime.date = estimates[0].date
    revenue_high: float = estimates[0].revenue_high
    eps_avg: float = estimates[0].eps_avg
    consensus: list[PriceTargetConsensus] = client.analyst.price_target_consensus("AAPL")
    target_high: float = consensus[0].target_high
    grades: list[StockGrade] = client.analyst.stock_grades("AAPL")
    graded_on: datetime.date = grades[0].date
    action: str = grades[0].action
    _ = (estimated_on, revenue_high, eps_avg, target_high, graded_on, action)


def check_tipranks_contract(client: FmpClient) -> None:
    """Type-check the flat tipranks namespace, its keyword-only filters and flag, and the nested count rows."""
    tipranks: TipranksNamespace = client.tipranks
    ratings: list[TipRanksRatingSearchResult] = tipranks.search_ratings(
        expert_uid="expert", symbol="RR.L", from_=datetime.date(2025, 6, 10), to="2026-06-10", limit=5000, page=0
    )
    recommended_on: datetime.date = ratings[0].recommendation_date
    currency: str = ratings[0].price_target_currency
    coverage: list[TipRanksPointInTimeRating] = client.tipranks.point_in_time_ratings_by_analyst(
        analyst_name="Keegan Cox", date=datetime.date(2026, 6, 10), nonadjusted=False
    )
    beat_target: bool | None = coverage[0].beat_target
    snapshot: list[TipRanksPointInTimeRating] = client.tipranks.point_in_time_ratings_by_symbol(
        "AAPL", date="2026-06-10"
    )
    last_recommended_on: datetime.date = snapshot[0].last_recommendation_date
    summaries: list[TipRanksAnalystSummary] = client.tipranks.analyst_summary("expert", from_="2025-06-10")
    summarised_from: datetime.date = summaries[0].from_
    counts: TipRanksRecommendationCounts = summaries[0].recommendations
    buys: int = counts.buy
    profiles: list[TipRanksAnalystProfile] = client.tipranks.analysts(page=0, limit=1000, firm_name="Roth MKM")
    stars: int = profiles[0].num_of_stars
    _ = (recommended_on, currency, beat_target, last_recommended_on, summarised_from, buys, stars)


def check_search_contract(client: FmpClient) -> None:
    """Type-check the flat search namespace, its keyword-only filters, and the single-identifier methods."""
    search: SearchNamespace = client.search
    matches: list[SymbolSearchResult] = search.symbol("Apple / Class A", limit=50, exchange="NASDAQ Global")
    exchange_full_name: str = matches[0].exchange_full_name
    entities: list[CikSearchResult] = client.search.cik("0000320193", limit=50)
    cik: str = entities[0].cik
    variants: list[ExchangeVariant] = client.search.exchange_variants("^VIX")
    ipo_date: datetime.date = variants[0].ipo_date
    market_cap: float = variants[0].market_cap
    is_etf: bool = variants[0].is_etf
    _ = (exchange_full_name, cik, ipo_date, market_cap, is_etf)


def check_insider_trading_contract(client: FmpClient) -> None:
    """Type-check the flat insider-trading namespace, its keyword-only filters, and the date rows."""
    insider_trading: InsiderTradingNamespace = client.insider_trading
    trades: list[InsiderTrade] = insider_trading.search_trades(
        symbol="AAPL",
        page=0,
        limit=100,
        reporting_cik="0001496686",
        company_cik="0000320193",
        transaction_type="S-Sale",
    )
    filed_on: datetime.date = trades[0].filing_date
    price: float = trades[0].price
    latest: list[InsiderTrade] = client.insider_trading.latest_trades(date=datetime.date(2026, 1, 27), page=0)
    traded_on: datetime.date = latest[0].transaction_date
    types: list[InsiderTransactionType] = client.insider_trading.transaction_types()
    code: str = types[0].transaction_type
    statistics: list[InsiderTradeStatistics] = client.insider_trading.trade_statistics("AAPL")
    ratio: float = statistics[0].acquired_disposed_ratio
    filings: list[BeneficialOwnershipAcquisition] = client.insider_trading.beneficial_ownership_acquisitions(
        "AAPL", limit=10
    )
    accepted_on: datetime.date = filings[0].accepted_date
    percent: str = filings[0].percent_of_class
    _ = (filed_on, price, traded_on, code, ratio, accepted_on, percent)


def check_fundraising_contract(client: FmpClient) -> None:
    """Type-check the flat fundraising namespace, its keyword-only filters, and the nullable date rows."""
    fundraising: FundraisingNamespace = client.fundraising
    crowdfunding: list[CrowdfundingOffering] = fundraising.latest_crowdfunding_offerings(page=0, limit=100)
    deadline: datetime.date = crowdfunding[0].offering_deadline_date
    accepted: datetime.datetime = crowdfunding[0].accepted_date
    price: Any = crowdfunding[0].offering_price
    by_cik: list[CrowdfundingOffering] = client.fundraising.crowdfunding_offerings_by_cik("0001916078")
    other_description: str | None = by_cik[0].security_offered_other_description
    campaigns: list[CrowdfundingOfferingSearchResult] = client.fundraising.search_crowdfunding_offerings("enotap")
    campaign_name: str = campaigns[0].name
    matches: list[RegulationDOfferingSearchResult] = client.fundraising.search_regulation_d_offerings("NJOY")
    matched_at: datetime.datetime = matches[0].date
    offerings: list[RegulationDOffering] = client.fundraising.latest_regulation_d_offerings(
        page=0, limit=10, cik="0002013736"
    )
    first_sale: datetime.date | None = offerings[0].date_of_first_sale
    recent: bool | None = offerings[0].incorporated_within_five_years
    issuer_rows: list[RegulationDOffering] = client.fundraising.regulation_d_offerings_by_cik("0001547416")
    sold: float = issuer_rows[0].total_amount_sold
    _ = (deadline, accepted, price, other_description, campaign_name, matched_at, first_sale, recent, sold)


def check_economics_contract(client: FmpClient) -> None:
    """Type-check the flat economics namespace, its required indicator name, the keyword-only filters, and the rows."""
    economics: EconomicsNamespace = client.economics
    rates: list[TreasuryRate] = economics.treasury_rates(from_=datetime.date(2026, 1, 27), to="2026-04-27")
    observed_on: datetime.date = rates[0].date
    year_30: float = rates[0].year_30
    observations: list[EconomicIndicatorObservation] = client.economics.indicators("GDP", from_="2025-04-27")
    name: str = observations[0].name
    value: float = observations[0].value
    events: list[EconomicCalendarEvent] = client.economics.calendar(
        country="US", from_="2026-01-27", to=datetime.date(2026, 4, 27)
    )
    event_time: datetime.datetime = events[0].date
    impact: str = events[0].impact
    premiums: list[MarketRiskPremium] = client.economics.market_risk_premium()
    premium: float = premiums[0].country_risk_premium
    _ = (observed_on, year_30, name, value, event_time, impact, premium)


def check_market_hours_contract(client: FmpClient) -> None:
    """Type-check the flat market-hours namespace, its required exchange, the keyword-only filters, and the rows."""
    market_hours: MarketHoursNamespace = client.market_hours
    hours: list[ExchangeMarketHours] = market_hours.exchange("NASDAQ", timestamp="001769527402")
    opening_hour: str = hours[0].opening_hour
    is_open: bool = hours[0].is_market_open
    holidays: list[ExchangeHoliday] = client.market_hours.holidays(
        "NASDAQ", from_=datetime.date(2025, 4, 27), to="2026-04-27"
    )
    holiday_date: datetime.date = holidays[0].date
    is_closed: bool = holidays[0].is_closed
    adj_open_time: Any = holidays[0].adj_open_time
    everywhere: list[ExchangeMarketHours] = client.market_hours.all_exchanges()
    timezone: str = everywhere[0].timezone
    _ = (opening_hour, is_open, holiday_date, is_closed, adj_open_time, timezone)


def check_commitment_of_traders_contract(client: FmpClient) -> None:
    """Type-check the flat COT namespace, its keyword-only symbol and date filters, and the query-less list."""
    commitment_of_traders: CommitmentOfTradersNamespace = client.commitment_of_traders
    reports: list[CotReport] = commitment_of_traders.report(
        symbol="VX", from_=datetime.date(2024, 1, 1), to="2024-03-01"
    )
    reported_on: datetime.datetime = reports[0].date
    spread_change: int = reports[0].change_in_noncomm_spread_all
    contract_units: str = reports[0].contract_units
    analyses: list[CotAnalysis] = client.commitment_of_traders.analysis(symbol="PA")
    net_position: int = analyses[0].net_position
    net_change: float = analyses[0].change_in_net_position
    reversal: bool = analyses[0].reversal_trend
    listings: list[CotReportListing] = client.commitment_of_traders.report_list()
    listed_symbol: str = listings[0].symbol
    _ = (reported_on, spread_change, contract_units, net_position, net_change, reversal, listed_symbol)


def check_esg_contract(client: FmpClient) -> None:
    """Type-check the flat esg namespace, its required symbol, the keyword-only year, and the rows."""
    esg: EsgNamespace = client.esg
    disclosures: list[EsgDisclosure] = esg.disclosures("AAPL")
    filed_on: datetime.date = disclosures[0].date
    accepted_on: datetime.date = disclosures[0].accepted_date
    esg_score: float = disclosures[0].esg_score
    ratings: list[EsgRating] = client.esg.ratings("AAPL")
    fiscal_year: int = ratings[0].fiscal_year
    risk_rating: str = ratings[0].esg_risk_rating
    benchmarks: list[EsgBenchmark] = client.esg.benchmark(year="FY 2024/25")
    all_years: list[EsgBenchmark] = client.esg.benchmark()
    sector: str = benchmarks[0].sector
    governance: float = all_years[0].governance_score
    _ = (filed_on, accepted_on, esg_score, fiscal_year, risk_rating, sector, governance)


def check_transcripts_contract(client: FmpClient) -> None:
    """Type-check the flat transcripts namespace, its required trio, the keyword-only options, and the rows."""
    transcripts: TranscriptsNamespace = client.transcripts
    latest: list[LatestEarningsTranscript] = transcripts.latest(limit=100, page=0)
    latest_period: str = latest[0].period
    latest_fiscal_year: int = latest[0].fiscal_year
    latest_date: datetime.date = latest[0].date
    full: list[EarningsTranscript] = client.transcripts.by_quarter("AAPL", 2020, 3, limit=1)
    year: int = full[0].year
    content: str = full[0].content
    held_on: datetime.date = full[0].date
    dates: list[EarningsTranscriptDate] = client.transcripts.dates("AAPL")
    quarter: int = dates[0].quarter
    available_on: datetime.date = dates[0].date
    _ = (latest_period, latest_fiscal_year, latest_date, year, content, held_on, quarter, available_on)


def check_dcf_contract(client: FmpClient) -> None:
    """Type-check the flat dcf namespace, the symbol-only methods, and the keyword-only flattened assumptions."""
    dcf: DcfNamespace = client.dcf
    standard: list[DcfValuation] = dcf.standard("AAPL")
    levered: list[DcfValuation] = client.dcf.levered("AAPL")
    valued_on: datetime.date = standard[0].date
    value: float = levered[0].dcf
    custom: list[CustomDcfValuation] = client.dcf.custom(
        "AAPL", revenue_growth_pct=0.109, tax_rate=0.149, long_term_growth_rate=4, beta=1.244
    )
    custom_levered: list[CustomLeveredDcfValuation] = client.dcf.custom_levered(
        "AAPL", cost_of_debt=3.64, cost_of_equity=9.51168, risk_free_rate=3.64
    )
    year: str = custom[0].year
    capital_expenditure: float = custom[0].capital_expenditure
    per_share: float = custom_levered[0].equity_value_per_share
    _ = (valued_on, value, year, capital_expenditure, per_share)


def check_public_contract(client: FmpClient) -> None:
    """Type-check the namespaced call shapes and the model fields."""
    short_rows: list[QuoteShort] = client.quote.short("AAPL")
    full_rows: list[Quote] = client.quote.full("AAPL")
    fund_rows: list[QuoteShort] = client.quote.mutual_funds()
    batch_rows: list[Quote] = client.quote.batch(["AAPL", "MSFT"])
    trades: list[AftermarketTrade] = client.quote.batch_aftermarket_trade("AAPL")
    changes: list[StockPriceChange] = client.quote.stock_price_change("AAPL")
    exchange_rows: list[QuoteShort] = client.quote.exchange("NASDAQ")
    trade_size: float = trades[0].trade_size
    ten_years: float = changes[0].ten_years
    symbol: str = short_rows[0].symbol
    price: float = full_rows[0].price
    volume: float = fund_rows[0].volume
    market_cap: float | None = full_rows[0].market_cap
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
