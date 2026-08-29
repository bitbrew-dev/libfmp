//! Institutional-ownership filing endpoints and query contracts.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata, GeographicAvailability},
    },
    query::{Quarter, Year},
    responses::institutional_ownership::{
        Form13fFilingDate, HolderIndustryBreakdown, HolderPerformanceSummary,
        InstitutionalHolderAnalytics, InstitutionalHolding, InstitutionalOwnershipFiling,
    },
    types::{Cik, Limit, Page, Ticker},
};

/// Optional page and limit for the latest institutional-ownership filings.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LatestInstitutionalOwnershipFilingsQuery {
    page: Option<Page>,
    limit: Option<Limit>,
}

impl LatestInstitutionalOwnershipFilingsQuery {
    /// Creates a query without undocumented page or limit defaults.
    pub const fn new() -> Self {
        Self {
            page: None,
            limit: None,
        }
    }

    /// Sets the optional provider page index.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }
}

impl QueryParameters for LatestInstitutionalOwnershipFilingsQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

/// Required filing identity for extracting one institutional holder's positions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstitutionalOwnershipExtractQuery {
    cik: Cik,
    year: Year,
    quarter: Quarter,
}

impl InstitutionalOwnershipExtractQuery {
    /// Creates a query for one CIK, query year, and textual query quarter.
    pub const fn new(cik: Cik, year: Year, quarter: Quarter) -> Self {
        Self { cik, year, quarter }
    }

    /// Borrows the required Central Index Key.
    pub const fn cik(&self) -> &Cik {
        &self.cik
    }

    /// Returns the required provider query year.
    pub const fn year(&self) -> Year {
        self.year
    }

    /// Returns the required textual provider query quarter.
    pub const fn quarter(&self) -> Quarter {
        self.quarter
    }
}

impl QueryParameters for InstitutionalOwnershipExtractQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("cik", &self.cik);
        encoder.required("year", self.year);
        encoder.required("quarter", self.quarter);
    }
}

/// Required CIK for retrieving available Form 13F filing dates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Form13fFilingDatesQuery {
    cik: Cik,
}

impl Form13fFilingDatesQuery {
    /// Creates a query for one Central Index Key.
    pub const fn new(cik: Cik) -> Self {
        Self { cik }
    }

    /// Borrows the required Central Index Key.
    pub const fn cik(&self) -> &Cik {
        &self.cik
    }
}

impl From<Cik> for Form13fFilingDatesQuery {
    fn from(cik: Cik) -> Self {
        Self::new(cik)
    }
}

impl From<&Cik> for Form13fFilingDatesQuery {
    fn from(cik: &Cik) -> Self {
        Self::new(cik.clone())
    }
}

impl QueryParameters for Form13fFilingDatesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("cik", &self.cik);
    }
}

/// Required security period and optional pagination for holder analytics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstitutionalHolderAnalyticsQuery {
    symbol: Ticker,
    year: Year,
    quarter: Quarter,
    page: Option<Page>,
    limit: Option<Limit>,
}

impl InstitutionalHolderAnalyticsQuery {
    /// Creates a query for one security and reporting period.
    pub fn new(symbol: Ticker, year: Year, quarter: Quarter) -> Self {
        Self {
            symbol,
            year,
            quarter,
            page: None,
            limit: None,
        }
    }

    /// Sets the optional provider page index.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Borrows the required security ticker.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }

    /// Returns the required provider query year.
    pub const fn year(&self) -> Year {
        self.year
    }

    /// Returns the required textual provider query quarter.
    pub const fn quarter(&self) -> Quarter {
        self.quarter
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }
}

impl QueryParameters for InstitutionalHolderAnalyticsQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.required("year", self.year);
        encoder.required("quarter", self.quarter);
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

/// Required holder CIK and optional pagination for performance summaries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderPerformanceSummaryQuery {
    cik: Cik,
    page: Option<Page>,
}

impl HolderPerformanceSummaryQuery {
    /// Creates a query for one Central Index Key without a page default.
    pub const fn new(cik: Cik) -> Self {
        Self { cik, page: None }
    }

    /// Sets the optional provider page index.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Borrows the required Central Index Key.
    pub const fn cik(&self) -> &Cik {
        &self.cik
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }
}

impl From<Cik> for HolderPerformanceSummaryQuery {
    fn from(cik: Cik) -> Self {
        Self::new(cik)
    }
}

impl From<&Cik> for HolderPerformanceSummaryQuery {
    fn from(cik: &Cik) -> Self {
        Self::new(cik.clone())
    }
}

impl QueryParameters for HolderPerformanceSummaryQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("cik", &self.cik);
        encoder.optional("page", self.page);
    }
}

/// Required holder CIK and reporting period for an industry breakdown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolderIndustryBreakdownQuery {
    cik: Cik,
    year: Year,
    quarter: Quarter,
}

impl HolderIndustryBreakdownQuery {
    /// Creates a query for one holder and reporting period.
    pub const fn new(cik: Cik, year: Year, quarter: Quarter) -> Self {
        Self { cik, year, quarter }
    }

    /// Borrows the required Central Index Key.
    pub const fn cik(&self) -> &Cik {
        &self.cik
    }

    /// Returns the required provider query year.
    pub const fn year(&self) -> Year {
        self.year
    }

    /// Returns the required textual provider query quarter.
    pub const fn quarter(&self) -> Quarter {
        self.quarter
    }
}

impl QueryParameters for HolderIndustryBreakdownQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("cik", &self.cik);
        encoder.required("year", self.year);
        encoder.required("quarter", self.quarter);
    }
}

const US_ONLY: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);
const LATEST_FILINGS_METADATA: EndpointMetadata =
    US_ONLY.with_bounds(EndpointBounds::new().with_page(100));

/// Describes `GET institutional-ownership/latest` without binding a transport.
pub fn latest_institutional_ownership_filings(
    query: LatestInstitutionalOwnershipFilingsQuery,
) -> EndpointSpec<LatestInstitutionalOwnershipFilingsQuery, Vec<InstitutionalOwnershipFiling>> {
    EndpointSpec::get(
        "institutional-ownership/latest",
        "institutional-ownership/latest",
        query,
    )
    .with_metadata(LATEST_FILINGS_METADATA)
}

/// Describes `GET institutional-ownership/extract` without binding a transport.
pub fn institutional_ownership_extract(
    query: InstitutionalOwnershipExtractQuery,
) -> EndpointSpec<InstitutionalOwnershipExtractQuery, Vec<InstitutionalHolding>> {
    EndpointSpec::get(
        "institutional-ownership/extract",
        "institutional-ownership/extract",
        query,
    )
    .with_metadata(US_ONLY)
}

/// Describes `GET institutional-ownership/dates` without binding a transport.
pub fn form_13f_filing_dates(
    query: Form13fFilingDatesQuery,
) -> EndpointSpec<Form13fFilingDatesQuery, Vec<Form13fFilingDate>> {
    EndpointSpec::get(
        "institutional-ownership/dates",
        "institutional-ownership/dates",
        query,
    )
    .with_metadata(US_ONLY)
}

/// Describes `GET institutional-ownership/extract-analytics/holder`.
pub fn institutional_holder_analytics(
    query: InstitutionalHolderAnalyticsQuery,
) -> EndpointSpec<InstitutionalHolderAnalyticsQuery, Vec<InstitutionalHolderAnalytics>> {
    EndpointSpec::get(
        "institutional-ownership/extract-analytics/holder",
        "institutional-ownership/extract-analytics/holder",
        query,
    )
    .with_metadata(US_ONLY)
}

/// Describes `GET institutional-ownership/holder-performance-summary`.
pub fn holder_performance_summary(
    query: HolderPerformanceSummaryQuery,
) -> EndpointSpec<HolderPerformanceSummaryQuery, Vec<HolderPerformanceSummary>> {
    EndpointSpec::get(
        "institutional-ownership/holder-performance-summary",
        "institutional-ownership/holder-performance-summary",
        query,
    )
    .with_metadata(US_ONLY)
}

/// Describes `GET institutional-ownership/holder-industry-breakdown`.
pub fn holder_industry_breakdown(
    query: HolderIndustryBreakdownQuery,
) -> EndpointSpec<HolderIndustryBreakdownQuery, Vec<HolderIndustryBreakdown>> {
    EndpointSpec::get(
        "institutional-ownership/holder-industry-breakdown",
        "institutional-ownership/holder-industry-breakdown",
        query,
    )
    .with_metadata(US_ONLY)
}

impl Client {
    /// Retrieves the latest US institutional-ownership filings.
    pub async fn latest_institutional_ownership_filings(
        &self,
        query: impl Into<LatestInstitutionalOwnershipFilingsQuery>,
    ) -> Result<Vec<InstitutionalOwnershipFiling>> {
        self.execute(&latest_institutional_ownership_filings(query.into()))
            .await
    }

    /// Extracts the positions from one US institutional-ownership filing period.
    pub async fn institutional_ownership_extract(
        &self,
        query: impl Into<InstitutionalOwnershipExtractQuery>,
    ) -> Result<Vec<InstitutionalHolding>> {
        self.execute(&institutional_ownership_extract(query.into()))
            .await
    }

    /// Retrieves available Form 13F filing dates for one US institutional holder.
    pub async fn form_13f_filing_dates(
        &self,
        query: impl Into<Form13fFilingDatesQuery>,
    ) -> Result<Vec<Form13fFilingDate>> {
        self.execute(&form_13f_filing_dates(query.into())).await
    }

    /// Retrieves holder-level analytics for one US security and filing period.
    pub async fn institutional_holder_analytics(
        &self,
        query: impl Into<InstitutionalHolderAnalyticsQuery>,
    ) -> Result<Vec<InstitutionalHolderAnalytics>> {
        self.execute(&institutional_holder_analytics(query.into()))
            .await
    }

    /// Retrieves a US institutional holder's portfolio performance summary.
    pub async fn holder_performance_summary(
        &self,
        query: impl Into<HolderPerformanceSummaryQuery>,
    ) -> Result<Vec<HolderPerformanceSummary>> {
        self.execute(&holder_performance_summary(query.into()))
            .await
    }

    /// Retrieves a US institutional holder's industry breakdown for one period.
    pub async fn holder_industry_breakdown(
        &self,
        query: impl Into<HolderIndustryBreakdownQuery>,
    ) -> Result<Vec<HolderIndustryBreakdown>> {
        self.execute(&holder_industry_breakdown(query.into())).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(query: &impl QueryParameters) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        query.encode(&mut QueryEncoder::new(&mut |name, value| {
            pairs.push((name.to_owned(), value.to_owned()));
        }));
        pairs
    }

    #[test]
    fn latest_query_omits_absent_values_and_encodes_page_then_limit() {
        assert!(pairs(&LatestInstitutionalOwnershipFilingsQuery::new()).is_empty());
        let query = LatestInstitutionalOwnershipFilingsQuery::new()
            .with_page(Page(0))
            .with_limit(Limit(100));
        assert_eq!(
            pairs(&query),
            [
                ("page".to_owned(), "0".to_owned()),
                ("limit".to_owned(), "100".to_owned()),
            ]
        );
    }

    #[test]
    fn extract_query_encodes_cik_year_then_quarter() {
        let query = InstitutionalOwnershipExtractQuery::new(
            Cik::new("0001388838").unwrap(),
            Year(2023),
            Quarter::Q3,
        );
        assert_eq!(
            pairs(&query),
            [
                ("cik".to_owned(), "0001388838".to_owned()),
                ("year".to_owned(), "2023".to_owned()),
                ("quarter".to_owned(), "3".to_owned()),
            ]
        );
    }

    #[test]
    fn dates_query_encodes_only_leading_zero_cik() {
        let query = Form13fFilingDatesQuery::new(Cik::new("0001067983").unwrap());
        assert_eq!(pairs(&query), [("cik".to_owned(), "0001067983".to_owned())]);
    }

    #[test]
    fn holder_analytics_encodes_symbol_year_quarter_page_then_limit() {
        let query = InstitutionalHolderAnalyticsQuery::new(
            Ticker::new("AAPL").unwrap(),
            Year(2023),
            Quarter::Q3,
        )
        .with_page(Page(0))
        .with_limit(Limit(10));
        assert_eq!(
            pairs(&query),
            [
                ("symbol".to_owned(), "AAPL".to_owned()),
                ("year".to_owned(), "2023".to_owned()),
                ("quarter".to_owned(), "3".to_owned()),
                ("page".to_owned(), "0".to_owned()),
                ("limit".to_owned(), "10".to_owned()),
            ]
        );
    }

    #[test]
    fn holder_summary_queries_encode_required_values_in_exact_order() {
        let cik = Cik::new("0001067983").unwrap();
        assert_eq!(
            pairs(&HolderPerformanceSummaryQuery::new(cik.clone()).with_page(Page(0))),
            [
                ("cik".to_owned(), "0001067983".to_owned()),
                ("page".to_owned(), "0".to_owned()),
            ]
        );
        assert_eq!(
            pairs(&HolderIndustryBreakdownQuery::new(
                cik,
                Year(2023),
                Quarter::Q3,
            )),
            [
                ("cik".to_owned(), "0001067983".to_owned()),
                ("year".to_owned(), "2023".to_owned()),
                ("quarter".to_owned(), "3".to_owned()),
            ]
        );
    }
}
