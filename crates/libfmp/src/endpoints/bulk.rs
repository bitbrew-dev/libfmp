//! Bulk endpoint contracts.
//!
//! A future Python binding reserves the matching `FmpClient` methods with
//! response models under `fmp.bulk`. This crate does not implement those bindings.
//!
//! Bulk descriptors and convenience methods inherit the client's finite
//! response-body limit. Raise [`crate::client::ClientBuilder::max_response_body_bytes`]
//! when a known provider response will exceed that configured limit.

use crate::{
    Client, Result,
    endpoints::{
        EndpointSpec, QueryEncoder, QueryParameters,
        metadata::{EndpointMetadata, GeographicAvailability},
    },
    query::{FiscalPeriod, Year},
    responses::{
        bulk::{
            BulkBalanceSheetStatement, BulkBalanceSheetStatementGrowth, BulkCashFlowStatement,
            BulkCashFlowStatementGrowth, BulkDcfValuation, BulkEarningsSurprise, BulkEodBar,
            BulkEtfHolding, BulkFinancialRatiosTtm, BulkFinancialScore, BulkIncomeStatement,
            BulkIncomeStatementGrowth, BulkKeyMetricsTtm, BulkPriceTargetSummary, BulkStockPeers,
            BulkStockRating, BulkUpgradesDowngradesConsensus,
        },
        company::CompanyProfile,
    },
    types::{BulkPart, Date},
};

/// Required provider partition shared by partitioned bulk routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BulkPartQuery {
    part: BulkPart,
}

impl BulkPartQuery {
    /// Creates a partitioned bulk query without inferring a numeric range.
    pub const fn new(part: BulkPart) -> Self {
        Self { part }
    }

    /// Borrows the original provider partition representation.
    pub const fn part(&self) -> &BulkPart {
        &self.part
    }
}

impl From<BulkPart> for BulkPartQuery {
    fn from(part: BulkPart) -> Self {
        Self::new(part)
    }
}

impl From<&BulkPart> for BulkPartQuery {
    fn from(part: &BulkPart) -> Self {
        Self::new(part.clone())
    }
}

impl QueryParameters for BulkPartQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("part", &self.part);
    }
}

/// Required provider year for the annual earnings-surprises bulk route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BulkYearQuery {
    year: Year,
}

impl BulkYearQuery {
    /// Creates a bulk year query without inferring an undocumented range.
    pub const fn new(year: Year) -> Self {
        Self { year }
    }

    /// Returns the requested provider year.
    pub const fn year(&self) -> Year {
        self.year
    }
}

impl From<Year> for BulkYearQuery {
    fn from(year: Year) -> Self {
        Self::new(year)
    }
}

impl QueryParameters for BulkYearQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("year", self.year);
    }
}

/// Required provider year and fiscal period shared by bulk statement routes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BulkStatementQuery {
    year: Year,
    period: FiscalPeriod,
}

impl BulkStatementQuery {
    /// Creates a bulk statement query with both documented required values.
    pub const fn new(year: Year, period: FiscalPeriod) -> Self {
        Self { year, period }
    }

    /// Returns the requested provider year.
    pub const fn year(&self) -> Year {
        self.year
    }

    /// Returns the requested fiscal period.
    pub const fn period(&self) -> FiscalPeriod {
        self.period
    }
}

impl QueryParameters for BulkStatementQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("year", self.year);
        encoder.required("period", self.period);
    }
}

/// Required provider date for the bulk end-of-day route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BulkEodQuery {
    date: Date,
}

impl BulkEodQuery {
    /// Creates a bulk end-of-day query for the documented required date.
    pub const fn new(date: Date) -> Self {
        Self { date }
    }

    /// Returns the requested provider date.
    pub const fn date(&self) -> Date {
        self.date
    }
}

impl From<Date> for BulkEodQuery {
    fn from(date: Date) -> Self {
        Self::new(date)
    }
}

impl QueryParameters for BulkEodQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("date", self.date);
    }
}

const WORLDWIDE: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);
const US_ONLY: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);

/// Describes `GET profile-bulk` without binding a transport.
pub fn bulk_company_profiles(
    query: BulkPartQuery,
) -> EndpointSpec<BulkPartQuery, Vec<CompanyProfile>> {
    EndpointSpec::get("profile-bulk", "profile-bulk", query).with_metadata(WORLDWIDE)
}

/// Describes `GET rating-bulk` without binding a transport.
pub fn bulk_stock_ratings() -> EndpointSpec<(), Vec<BulkStockRating>> {
    EndpointSpec::get("rating-bulk", "rating-bulk", ()).with_metadata(WORLDWIDE)
}

/// Describes `GET dcf-bulk` without binding a transport.
pub fn bulk_dcf_valuations() -> EndpointSpec<(), Vec<BulkDcfValuation>> {
    EndpointSpec::get("dcf-bulk", "dcf-bulk", ()).with_metadata(WORLDWIDE)
}

/// Describes `GET scores-bulk` without binding a transport.
pub fn bulk_financial_scores() -> EndpointSpec<(), Vec<BulkFinancialScore>> {
    EndpointSpec::get("scores-bulk", "scores-bulk", ()).with_metadata(WORLDWIDE)
}

/// Describes `GET price-target-summary-bulk` without binding a transport.
pub fn bulk_price_target_summaries() -> EndpointSpec<(), Vec<BulkPriceTargetSummary>> {
    EndpointSpec::get("price-target-summary-bulk", "price-target-summary-bulk", ())
        .with_metadata(US_ONLY)
}

/// Describes `GET etf-holder-bulk` without binding a transport.
pub fn bulk_etf_holdings(query: BulkPartQuery) -> EndpointSpec<BulkPartQuery, Vec<BulkEtfHolding>> {
    EndpointSpec::get("etf-holder-bulk", "etf-holder-bulk", query).with_metadata(WORLDWIDE)
}

/// Describes `GET upgrades-downgrades-consensus-bulk` without binding a transport.
pub fn bulk_upgrades_downgrades_consensus() -> EndpointSpec<(), Vec<BulkUpgradesDowngradesConsensus>>
{
    EndpointSpec::get(
        "upgrades-downgrades-consensus-bulk",
        "upgrades-downgrades-consensus-bulk",
        (),
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET key-metrics-ttm-bulk` without binding a transport.
pub fn bulk_key_metrics_ttm() -> EndpointSpec<(), Vec<BulkKeyMetricsTtm>> {
    EndpointSpec::get("key-metrics-ttm-bulk", "key-metrics-ttm-bulk", ()).with_metadata(WORLDWIDE)
}

/// Describes `GET ratios-ttm-bulk` without binding a transport.
pub fn bulk_financial_ratios_ttm() -> EndpointSpec<(), Vec<BulkFinancialRatiosTtm>> {
    EndpointSpec::get("ratios-ttm-bulk", "ratios-ttm-bulk", ()).with_metadata(WORLDWIDE)
}

/// Describes `GET peers-bulk` without binding a transport.
pub fn bulk_stock_peers() -> EndpointSpec<(), Vec<BulkStockPeers>> {
    EndpointSpec::get("peers-bulk", "peers-bulk", ()).with_metadata(WORLDWIDE)
}

/// Describes `GET earnings-surprises-bulk` without binding a transport.
pub fn bulk_earnings_surprises(
    query: BulkYearQuery,
) -> EndpointSpec<BulkYearQuery, Vec<BulkEarningsSurprise>> {
    EndpointSpec::get("earnings-surprises-bulk", "earnings-surprises-bulk", query)
        .with_metadata(WORLDWIDE)
}

/// Describes `GET income-statement-bulk` without binding a transport.
pub fn bulk_income_statements(
    query: BulkStatementQuery,
) -> EndpointSpec<BulkStatementQuery, Vec<BulkIncomeStatement>> {
    EndpointSpec::get("income-statement-bulk", "income-statement-bulk", query)
        .with_metadata(WORLDWIDE)
}

/// Describes `GET income-statement-growth-bulk` without binding a transport.
pub fn bulk_income_statement_growth(
    query: BulkStatementQuery,
) -> EndpointSpec<BulkStatementQuery, Vec<BulkIncomeStatementGrowth>> {
    EndpointSpec::get(
        "income-statement-growth-bulk",
        "income-statement-growth-bulk",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET balance-sheet-statement-bulk` without binding a transport.
pub fn bulk_balance_sheet_statements(
    query: BulkStatementQuery,
) -> EndpointSpec<BulkStatementQuery, Vec<BulkBalanceSheetStatement>> {
    EndpointSpec::get(
        "balance-sheet-statement-bulk",
        "balance-sheet-statement-bulk",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET balance-sheet-statement-growth-bulk` without binding a transport.
pub fn bulk_balance_sheet_statement_growth(
    query: BulkStatementQuery,
) -> EndpointSpec<BulkStatementQuery, Vec<BulkBalanceSheetStatementGrowth>> {
    EndpointSpec::get(
        "balance-sheet-statement-growth-bulk",
        "balance-sheet-statement-growth-bulk",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET cash-flow-statement-bulk` without binding a transport.
pub fn bulk_cash_flow_statements(
    query: BulkStatementQuery,
) -> EndpointSpec<BulkStatementQuery, Vec<BulkCashFlowStatement>> {
    EndpointSpec::get(
        "cash-flow-statement-bulk",
        "cash-flow-statement-bulk",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET cash-flow-statement-growth-bulk` without binding a transport.
pub fn bulk_cash_flow_statement_growth(
    query: BulkStatementQuery,
) -> EndpointSpec<BulkStatementQuery, Vec<BulkCashFlowStatementGrowth>> {
    EndpointSpec::get(
        "cash-flow-statement-growth-bulk",
        "cash-flow-statement-growth-bulk",
        query,
    )
    .with_metadata(WORLDWIDE)
}

/// Describes `GET eod-bulk` without binding a transport.
pub fn bulk_eod(query: BulkEodQuery) -> EndpointSpec<BulkEodQuery, Vec<BulkEodBar>> {
    EndpointSpec::get("eod-bulk", "eod-bulk", query).with_metadata(WORLDWIDE)
}

impl Client {
    /// Retrieves one provider partition of worldwide company profiles.
    pub async fn bulk_company_profiles(
        &self,
        query: impl Into<BulkPartQuery>,
    ) -> Result<Vec<CompanyProfile>> {
        self.execute(&bulk_company_profiles(query.into())).await
    }

    /// Retrieves worldwide stock ratings in one provider bulk response.
    pub async fn bulk_stock_ratings(&self) -> Result<Vec<BulkStockRating>> {
        self.execute(&bulk_stock_ratings()).await
    }

    /// Retrieves worldwide discounted-cash-flow valuations in one provider bulk response.
    pub async fn bulk_dcf_valuations(&self) -> Result<Vec<BulkDcfValuation>> {
        self.execute(&bulk_dcf_valuations()).await
    }

    /// Retrieves worldwide financial scores in one provider bulk response.
    pub async fn bulk_financial_scores(&self) -> Result<Vec<BulkFinancialScore>> {
        self.execute(&bulk_financial_scores()).await
    }

    /// Retrieves US price-target summaries in one provider bulk response.
    pub async fn bulk_price_target_summaries(&self) -> Result<Vec<BulkPriceTargetSummary>> {
        self.execute(&bulk_price_target_summaries()).await
    }

    /// Retrieves one provider partition of worldwide ETF holdings.
    pub async fn bulk_etf_holdings(
        &self,
        query: impl Into<BulkPartQuery>,
    ) -> Result<Vec<BulkEtfHolding>> {
        self.execute(&bulk_etf_holdings(query.into())).await
    }

    /// Retrieves worldwide upgrades/downgrades consensus in one provider bulk response.
    pub async fn bulk_upgrades_downgrades_consensus(
        &self,
    ) -> Result<Vec<BulkUpgradesDowngradesConsensus>> {
        self.execute(&bulk_upgrades_downgrades_consensus()).await
    }

    /// Retrieves worldwide trailing-twelve-month key metrics in one bulk response.
    pub async fn bulk_key_metrics_ttm(&self) -> Result<Vec<BulkKeyMetricsTtm>> {
        self.execute(&bulk_key_metrics_ttm()).await
    }

    /// Retrieves worldwide trailing-twelve-month financial ratios in one bulk response.
    pub async fn bulk_financial_ratios_ttm(&self) -> Result<Vec<BulkFinancialRatiosTtm>> {
        self.execute(&bulk_financial_ratios_ttm()).await
    }

    /// Retrieves worldwide stock peers in one provider bulk response.
    pub async fn bulk_stock_peers(&self) -> Result<Vec<BulkStockPeers>> {
        self.execute(&bulk_stock_peers()).await
    }

    /// Retrieves worldwide annual earnings surprises for one required provider year.
    pub async fn bulk_earnings_surprises(
        &self,
        query: impl Into<BulkYearQuery>,
    ) -> Result<Vec<BulkEarningsSurprise>> {
        self.execute(&bulk_earnings_surprises(query.into())).await
    }

    /// Retrieves worldwide bulk income statements for one year and fiscal period.
    pub async fn bulk_income_statements(
        &self,
        query: BulkStatementQuery,
    ) -> Result<Vec<BulkIncomeStatement>> {
        self.execute(&bulk_income_statements(query)).await
    }

    /// Retrieves worldwide bulk income-statement growth for one year and fiscal period.
    pub async fn bulk_income_statement_growth(
        &self,
        query: BulkStatementQuery,
    ) -> Result<Vec<BulkIncomeStatementGrowth>> {
        self.execute(&bulk_income_statement_growth(query)).await
    }

    /// Retrieves worldwide bulk balance sheets for one year and fiscal period.
    pub async fn bulk_balance_sheet_statements(
        &self,
        query: BulkStatementQuery,
    ) -> Result<Vec<BulkBalanceSheetStatement>> {
        self.execute(&bulk_balance_sheet_statements(query)).await
    }

    /// Retrieves worldwide bulk balance-sheet growth for one year and fiscal period.
    pub async fn bulk_balance_sheet_statement_growth(
        &self,
        query: BulkStatementQuery,
    ) -> Result<Vec<BulkBalanceSheetStatementGrowth>> {
        self.execute(&bulk_balance_sheet_statement_growth(query))
            .await
    }

    /// Retrieves worldwide bulk cash-flow statements for one year and fiscal period.
    pub async fn bulk_cash_flow_statements(
        &self,
        query: BulkStatementQuery,
    ) -> Result<Vec<BulkCashFlowStatement>> {
        self.execute(&bulk_cash_flow_statements(query)).await
    }

    /// Retrieves worldwide bulk cash-flow growth for one year and fiscal period.
    pub async fn bulk_cash_flow_statement_growth(
        &self,
        query: BulkStatementQuery,
    ) -> Result<Vec<BulkCashFlowStatementGrowth>> {
        self.execute(&bulk_cash_flow_statement_growth(query)).await
    }

    /// Retrieves worldwide bulk end-of-day prices for one required date.
    pub async fn bulk_eod(&self, query: impl Into<BulkEodQuery>) -> Result<Vec<BulkEodBar>> {
        self.execute(&bulk_eod(query.into())).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encoded(query: &impl QueryParameters) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        let mut visitor = |name: &str, value: &str| {
            pairs.push((name.to_owned(), value.to_owned()));
        };
        query.encode(&mut QueryEncoder::new(&mut visitor));
        pairs
    }

    #[test]
    fn bulk_part_query_has_one_exact_required_key_and_borrowed_conversion() {
        let part = BulkPart::new("segment 01/alpha").unwrap();
        let owned = BulkPartQuery::from(part.clone());
        let borrowed = BulkPartQuery::from(&part);

        assert_eq!(owned.part(), &part);
        assert_eq!(borrowed.part(), &part);
        assert_eq!(
            encoded(&borrowed),
            [("part".into(), "segment 01/alpha".into())]
        );
    }

    #[test]
    fn bulk_year_query_has_one_exact_required_key() {
        let query = BulkYearQuery::new(Year(2026));

        assert_eq!(query.year(), Year(2026));
        assert_eq!(encoded(&query), [("year".into(), "2026".into())]);
        assert_eq!(BulkYearQuery::from(Year(7)), BulkYearQuery::new(Year(7)));
    }

    #[test]
    fn bulk_statement_query_has_two_exact_required_keys_in_documented_order() {
        let query = BulkStatementQuery::new(Year(2026), FiscalPeriod::Q1);

        assert_eq!(query.year(), Year(2026));
        assert_eq!(query.period(), FiscalPeriod::Q1);
        assert_eq!(
            encoded(&query),
            [
                ("year".into(), "2026".into()),
                ("period".into(), "Q1".into())
            ]
        );
    }

    #[test]
    fn bulk_eod_query_has_one_exact_required_key() {
        let date = Date::parse("2024-10-22").unwrap();
        let query = BulkEodQuery::new(date);

        assert_eq!(query.date(), date);
        assert_eq!(encoded(&query), [("date".into(), "2024-10-22".into())]);
        assert_eq!(BulkEodQuery::from(date), query);
    }

    #[test]
    fn bulk_descriptors_inherit_the_finite_client_response_limit() {
        assert_eq!(bulk_stock_ratings().max_response_body_bytes(), None);
        assert_eq!(bulk_key_metrics_ttm().max_response_body_bytes(), None);
        assert_eq!(bulk_financial_ratios_ttm().max_response_body_bytes(), None);
    }
}
