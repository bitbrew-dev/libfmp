use libfmp::{
    endpoints::insider_trading::{
        BeneficialOwnershipAcquisitionsQuery, InsiderReportingNameSearchQuery,
        InsiderTradeStatisticsQuery, InsiderTradesSearchQuery, LatestInsiderTradesQuery,
    },
    types::{Cik, Date, Limit, Page, SearchTerm, Ticker, TransactionTypeCode},
};

#[test]
fn latest_query_preserves_omission_zero_and_full_numeric_domains() {
    let empty = LatestInsiderTradesQuery::new();
    assert_eq!(empty.date(), None);
    assert_eq!(empty.page(), None);
    assert_eq!(empty.limit(), None);

    let full = empty
        .with_date(Date::parse("2026-01-27").unwrap())
        .with_page(Page(0))
        .with_limit(Limit(u32::MAX));
    assert_eq!(full.date(), Some(Date::parse("2026-01-27").unwrap()));
    assert_eq!(full.page(), Some(Page(0)));
    assert_eq!(full.limit(), Some(Limit(u32::MAX)));
}

#[test]
fn trade_search_keeps_every_filter_independently_optional() {
    let empty = InsiderTradesSearchQuery::new();
    assert_eq!(empty.symbol(), None);
    assert_eq!(empty.page(), None);
    assert_eq!(empty.limit(), None);
    assert_eq!(empty.reporting_cik(), None);
    assert_eq!(empty.company_cik(), None);
    assert_eq!(empty.transaction_type(), None);

    let query = empty
        .with_symbol(Ticker::new("AAPL").unwrap())
        .with_page(Page(u32::MAX))
        .with_limit(Limit(0))
        .with_reporting_cik(Cik::new("0001496686").unwrap())
        .with_company_cik(Cik::new("0000320193").unwrap())
        .with_transaction_type(TransactionTypeCode::new("S-Sale").unwrap());
    assert_eq!(query.symbol().unwrap().as_str(), "AAPL");
    assert_eq!(query.page(), Some(Page(u32::MAX)));
    assert_eq!(query.limit(), Some(Limit(0)));
    assert_eq!(query.reporting_cik().unwrap().as_str(), "0001496686");
    assert_eq!(query.company_cik().unwrap().as_str(), "0000320193");
    assert_eq!(query.transaction_type().unwrap().as_str(), "S-Sale");
}

#[test]
fn required_queries_preserve_exact_text_tickers_and_optional_limit() {
    let term = SearchTerm::new("  Zuckerberg, Mark  ").unwrap();
    let name: InsiderReportingNameSearchQuery = (&term).into();
    assert_eq!(name.name(), &term);

    let ticker = Ticker::new("000001.SZ").unwrap();
    let statistics: InsiderTradeStatisticsQuery = (&ticker).into();
    assert_eq!(statistics.symbol(), &ticker);

    let ownership = BeneficialOwnershipAcquisitionsQuery::new(ticker).with_limit(Limit(u32::MAX));
    assert_eq!(ownership.symbol().as_str(), "000001.SZ");
    assert_eq!(ownership.limit(), Some(Limit(u32::MAX)));
}
