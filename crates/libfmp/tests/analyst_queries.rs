use libfmp::{
    endpoints::analyst::{
        FinancialEstimatesQuery, HistoricalRatingsQuery, HistoricalStockGradesQuery,
        PriceTargetConsensusQuery, PriceTargetSummaryQuery, RatingsSnapshotQuery, StockGradesQuery,
        StockGradesSummaryQuery,
    },
    query::RetrievalFrequency,
    types::{Limit, Page, Ticker},
};

#[test]
fn estimates_require_the_narrow_frequency_and_preserve_unbounded_pagination() {
    let symbol = Ticker::new("BRK.B / Class A").unwrap();
    let annual = FinancialEstimatesQuery::new(symbol.clone(), RetrievalFrequency::Annual);
    assert_eq!(annual.symbol(), &symbol);
    assert_eq!(annual.period(), RetrievalFrequency::Annual);
    assert_eq!(annual.period().to_string(), "annual");
    assert_eq!(annual.page(), None);
    assert_eq!(annual.limit(), None);

    let quarterly = FinancialEstimatesQuery::new(symbol, RetrievalFrequency::Quarterly)
        .with_page(Page(0))
        .with_limit(Limit(u32::MAX));
    assert_eq!(quarterly.period().to_string(), "quarter");
    assert_eq!(quarterly.page(), Some(Page(0)));
    assert_eq!(quarterly.limit(), Some(Limit(u32::MAX)));
}

#[test]
fn every_symbol_only_query_preserves_owned_and_borrowed_tickers() {
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    macro_rules! assert_symbol_query {
        ($query:ty) => {{
            let owned: $query = symbol.clone().into();
            assert_eq!(owned.symbol(), &symbol);
            let borrowed: $query = (&symbol).into();
            assert_eq!(borrowed.symbol(), &symbol);
        }};
    }

    assert_symbol_query!(RatingsSnapshotQuery);
    assert_symbol_query!(PriceTargetSummaryQuery);
    assert_symbol_query!(PriceTargetConsensusQuery);
    assert_symbol_query!(StockGradesQuery);
    assert_symbol_query!(StockGradesSummaryQuery);
}

#[test]
fn both_historical_queries_preserve_omission_zero_and_full_limit_domain() {
    let symbol = Ticker::new("AAPL").unwrap();

    let ratings: HistoricalRatingsQuery = (&symbol).into();
    assert_eq!(ratings.symbol(), &symbol);
    assert_eq!(ratings.limit(), None);
    assert_eq!(ratings.with_limit(Limit(0)).limit(), Some(Limit(0)));

    let grades: HistoricalStockGradesQuery = symbol.into();
    assert_eq!(grades.limit(), None);
    assert_eq!(
        grades.with_limit(Limit(u32::MAX)).limit(),
        Some(Limit(u32::MAX))
    );
}
