use libfmp::{
    endpoints::institutional_ownership::{
        Form13fFilingDatesQuery, HolderIndustryBreakdownQuery, HolderPerformanceSummaryQuery,
        InstitutionalIndustrySummaryQuery, InstitutionalOwnershipExtractQuery,
        InstitutionalPositionsSummaryQuery, LatestInstitutionalOwnershipFilingsQuery,
    },
    query::{Quarter, Year},
    types::{Cik, Limit, Page},
};

#[test]
fn latest_query_preserves_omission_zero_documented_and_unenforced_bounds() {
    let empty = LatestInstitutionalOwnershipFilingsQuery::new();
    assert_eq!(empty.page(), None);
    assert_eq!(empty.limit(), None);

    for page in [Page(0), Page(100), Page(101)] {
        assert_eq!(empty.with_page(page).page(), Some(page));
    }
    assert_eq!(
        empty.with_limit(Limit(u32::MAX)).limit(),
        Some(Limit(u32::MAX))
    );
}

#[test]
fn extract_query_reuses_leading_zero_cik_year_and_textual_quarter() {
    let cik = Cik::new("0001388838").unwrap();
    let query = InstitutionalOwnershipExtractQuery::new(cik.clone(), Year(2023), Quarter::Q3);
    assert_eq!(query.cik(), &cik);
    assert_eq!(query.cik().as_str(), "0001388838");
    assert_eq!(query.year(), Year(2023));
    assert_eq!(query.quarter(), Quarter::Q3);
}

#[test]
fn dates_query_preserves_owned_and_borrowed_cik_conversions() {
    let cik = Cik::new("0001067983").unwrap();
    let owned: Form13fFilingDatesQuery = cik.clone().into();
    assert_eq!(owned.cik(), &cik);

    let borrowed: Form13fFilingDatesQuery = (&cik).into();
    assert_eq!(borrowed.cik(), &cik);
    assert_eq!(borrowed.cik().as_str(), "0001067983");
}

#[test]
fn holder_performance_query_preserves_cik_conversions_and_unbounded_page() {
    let cik = Cik::new("0001067983").unwrap();
    let owned: HolderPerformanceSummaryQuery = cik.clone().into();
    assert_eq!(owned.cik(), &cik);
    assert_eq!(owned.page(), None);

    let borrowed: HolderPerformanceSummaryQuery = (&cik).into();
    assert_eq!(borrowed.cik().as_str(), "0001067983");
    assert_eq!(
        borrowed.with_page(Page(u32::MAX)).page(),
        Some(Page(u32::MAX))
    );
}

#[test]
fn holder_industry_query_reuses_leading_zero_cik_year_and_textual_quarter() {
    let cik = Cik::new("0001067983").unwrap();
    let query = HolderIndustryBreakdownQuery::new(cik.clone(), Year(2023), Quarter::Q3);
    assert_eq!(query.cik(), &cik);
    assert_eq!(query.cik().as_str(), "0001067983");
    assert_eq!(query.year(), Year(2023));
    assert_eq!(query.quarter(), Quarter::Q3);
}

#[test]
fn position_and_industry_summary_queries_preserve_required_fundamentals() {
    let symbol = libfmp::types::Ticker::new("BRK.B / Class A").unwrap();
    let positions =
        InstitutionalPositionsSummaryQuery::new(symbol.clone(), Year(2023), Quarter::Q3);
    assert_eq!(positions.symbol(), &symbol);
    assert_eq!(positions.year(), Year(2023));
    assert_eq!(positions.quarter(), Quarter::Q3);

    let industry = InstitutionalIndustrySummaryQuery::new(Year(2024), Quarter::Q4);
    assert_eq!(industry.year(), Year(2024));
    assert_eq!(industry.quarter(), Quarter::Q4);
}
