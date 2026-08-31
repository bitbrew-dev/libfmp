//! Insider-trading endpoint query contracts.

use crate::{
    endpoints::{QueryEncoder, QueryParameters},
    types::{Cik, Date, Limit, Page, SearchTerm, Ticker, TransactionTypeCode},
};

/// Optional date and pagination filters for the latest insider trades.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LatestInsiderTradesQuery {
    date: Option<Date>,
    page: Option<Page>,
    limit: Option<Limit>,
}

impl LatestInsiderTradesQuery {
    /// Creates a query without undocumented defaults.
    pub const fn new() -> Self {
        Self {
            date: None,
            page: None,
            limit: None,
        }
    }

    /// Sets the optional provider date filter.
    pub const fn with_date(mut self, date: Date) -> Self {
        self.date = Some(date);
        self
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

    /// Returns the optional provider date filter.
    pub const fn date(&self) -> Option<Date> {
        self.date
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

impl QueryParameters for LatestInsiderTradesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("date", self.date);
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

/// Optional filters for searching insider trades.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InsiderTradesSearchQuery {
    symbol: Option<Ticker>,
    page: Option<Page>,
    limit: Option<Limit>,
    reporting_cik: Option<Cik>,
    company_cik: Option<Cik>,
    transaction_type: Option<TransactionTypeCode>,
}

impl InsiderTradesSearchQuery {
    /// Creates a search without undocumented defaults.
    pub const fn new() -> Self {
        Self {
            symbol: None,
            page: None,
            limit: None,
            reporting_cik: None,
            company_cik: None,
            transaction_type: None,
        }
    }

    /// Sets the optional ticker filter.
    pub fn with_symbol(mut self, symbol: Ticker) -> Self {
        self.symbol = Some(symbol);
        self
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

    /// Sets the optional reporting-person CIK filter.
    pub fn with_reporting_cik(mut self, reporting_cik: Cik) -> Self {
        self.reporting_cik = Some(reporting_cik);
        self
    }

    /// Sets the optional company CIK filter.
    pub fn with_company_cik(mut self, company_cik: Cik) -> Self {
        self.company_cik = Some(company_cik);
        self
    }

    /// Sets the optional open transaction-type filter.
    pub fn with_transaction_type(mut self, transaction_type: TransactionTypeCode) -> Self {
        self.transaction_type = Some(transaction_type);
        self
    }

    /// Borrows the optional ticker filter.
    pub const fn symbol(&self) -> Option<&Ticker> {
        self.symbol.as_ref()
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }

    /// Borrows the optional reporting-person CIK filter.
    pub const fn reporting_cik(&self) -> Option<&Cik> {
        self.reporting_cik.as_ref()
    }

    /// Borrows the optional company CIK filter.
    pub const fn company_cik(&self) -> Option<&Cik> {
        self.company_cik.as_ref()
    }

    /// Borrows the optional transaction-type filter.
    pub const fn transaction_type(&self) -> Option<&TransactionTypeCode> {
        self.transaction_type.as_ref()
    }
}

impl QueryParameters for InsiderTradesSearchQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("symbol", self.symbol.as_ref());
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
        encoder.optional("reportingCik", self.reporting_cik.as_ref());
        encoder.optional("companyCik", self.company_cik.as_ref());
        encoder.optional("transactionType", self.transaction_type.as_ref());
    }
}

/// Required reporting-name search text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsiderReportingNameSearchQuery {
    name: SearchTerm,
}

impl InsiderReportingNameSearchQuery {
    /// Creates a reporting-name search.
    pub const fn new(name: SearchTerm) -> Self {
        Self { name }
    }

    /// Borrows the exact search text.
    pub const fn name(&self) -> &SearchTerm {
        &self.name
    }
}

impl From<SearchTerm> for InsiderReportingNameSearchQuery {
    fn from(name: SearchTerm) -> Self {
        Self::new(name)
    }
}

impl From<&SearchTerm> for InsiderReportingNameSearchQuery {
    fn from(name: &SearchTerm) -> Self {
        Self::new(name.clone())
    }
}

impl QueryParameters for InsiderReportingNameSearchQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("name", &self.name);
    }
}

/// Required ticker for insider-trade statistics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsiderTradeStatisticsQuery {
    symbol: Ticker,
}

impl InsiderTradeStatisticsQuery {
    /// Creates a statistics query for one ticker.
    pub const fn new(symbol: Ticker) -> Self {
        Self { symbol }
    }

    /// Borrows the requested ticker.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }
}

impl From<Ticker> for InsiderTradeStatisticsQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for InsiderTradeStatisticsQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for InsiderTradeStatisticsQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
    }
}

/// Required ticker and optional limit for beneficial-ownership acquisitions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BeneficialOwnershipAcquisitionsQuery {
    symbol: Ticker,
    limit: Option<Limit>,
}

impl BeneficialOwnershipAcquisitionsQuery {
    /// Creates a query without an undocumented limit.
    pub const fn new(symbol: Ticker) -> Self {
        Self {
            symbol,
            limit: None,
        }
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Borrows the requested ticker.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }
}

impl From<Ticker> for BeneficialOwnershipAcquisitionsQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for BeneficialOwnershipAcquisitionsQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for BeneficialOwnershipAcquisitionsQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.optional("limit", self.limit);
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
    fn latest_query_omits_all_filters_and_preserves_full_domains() {
        assert!(encoded(&LatestInsiderTradesQuery::new()).is_empty());
        let query = LatestInsiderTradesQuery::new()
            .with_date(Date::parse("2026-01-27").unwrap())
            .with_page(Page(0))
            .with_limit(Limit(u32::MAX));
        assert_eq!(
            encoded(&query),
            [
                ("date".into(), "2026-01-27".into()),
                ("page".into(), "0".into()),
                ("limit".into(), u32::MAX.to_string()),
            ]
        );
    }

    #[test]
    fn search_query_emits_exact_keys_in_documented_order() {
        assert!(encoded(&InsiderTradesSearchQuery::new()).is_empty());
        let query = InsiderTradesSearchQuery::new()
            .with_symbol(Ticker::new("AAPL").unwrap())
            .with_page(Page(u32::MAX))
            .with_limit(Limit(0))
            .with_reporting_cik(Cik::new("0001496686").unwrap())
            .with_company_cik(Cik::new("0000320193").unwrap())
            .with_transaction_type(TransactionTypeCode::new("S-Sale / future").unwrap());
        assert_eq!(
            encoded(&query),
            [
                ("symbol".into(), "AAPL".into()),
                ("page".into(), u32::MAX.to_string()),
                ("limit".into(), "0".into()),
                ("reportingCik".into(), "0001496686".into()),
                ("companyCik".into(), "0000320193".into()),
                ("transactionType".into(), "S-Sale / future".into()),
            ]
        );
    }

    #[test]
    fn required_queries_and_unit_taxonomy_query_have_exact_shapes() {
        let name =
            InsiderReportingNameSearchQuery::new(SearchTerm::new("Zuckerberg, Mark").unwrap());
        assert_eq!(encoded(&name), [("name".into(), "Zuckerberg, Mark".into())]);

        let statistics = InsiderTradeStatisticsQuery::new(Ticker::new("AAPL").unwrap());
        assert_eq!(encoded(&statistics), [("symbol".into(), "AAPL".into())]);

        let ownership =
            BeneficialOwnershipAcquisitionsQuery::new(Ticker::new("000001.SZ").unwrap())
                .with_limit(Limit(u32::MAX));
        assert_eq!(
            encoded(&ownership),
            [
                ("symbol".into(), "000001.SZ".into()),
                ("limit".into(), u32::MAX.to_string()),
            ]
        );

        assert!(encoded(&()).is_empty());
    }
}
