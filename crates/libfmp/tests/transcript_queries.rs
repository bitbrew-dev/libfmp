use libfmp::{
    endpoints::transcripts::{
        EarningsTranscriptDatesQuery, EarningsTranscriptQuery, LatestEarningsTranscriptsQuery,
    },
    query::{Quarter, Year},
    types::{Limit, Page, Ticker},
};

#[test]
fn latest_query_preserves_omission_limit_and_page_zero() {
    let empty = LatestEarningsTranscriptsQuery::new();
    assert_eq!(empty.limit(), None);
    assert_eq!(empty.page(), None);

    let query = empty.with_limit(Limit(100)).with_page(Page(0));
    assert_eq!(query.limit(), Some(Limit(100)));
    assert_eq!(query.page(), Some(Page(0)));

    let metadata_owned_bounds = LatestEarningsTranscriptsQuery::new()
        .with_limit(Limit(101))
        .with_page(Page(101));
    assert_eq!(metadata_owned_bounds.limit(), Some(Limit(101)));
    assert_eq!(metadata_owned_bounds.page(), Some(Page(101)));
}

#[test]
fn transcript_query_reuses_year_and_textual_quarter_query_fundamentals() {
    let symbol = Ticker::new("AAPL").unwrap();
    let query = EarningsTranscriptQuery::new(symbol.clone(), Year(2020), Quarter::Q3);
    assert_eq!(query.symbol(), &symbol);
    assert_eq!(query.year(), Year(2020));
    assert_eq!(query.quarter(), Quarter::Q3);
    assert_eq!(query.limit(), None);
    assert_eq!(query.with_limit(Limit(1)).limit(), Some(Limit(1)));
}

#[test]
fn transcript_dates_query_preserves_owned_and_borrowed_ticker_conversions() {
    let symbol = Ticker::new("AAPL").unwrap();
    let borrowed: EarningsTranscriptDatesQuery = (&symbol).into();
    assert_eq!(borrowed.symbol(), &symbol);

    let owned: EarningsTranscriptDatesQuery = symbol.clone().into();
    assert_eq!(owned.symbol(), &symbol);
}
