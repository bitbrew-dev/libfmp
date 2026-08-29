//! Earnings-transcript endpoint query contracts.

pub use crate::endpoints::directory::earnings_transcript_list;

use crate::{
    endpoints::{QueryEncoder, QueryParameters},
    query::{Quarter, Year},
    types::{Limit, Page, Ticker},
};

/// Optional limit and page for the latest earnings-transcript metadata.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LatestEarningsTranscriptsQuery {
    limit: Option<Limit>,
    page: Option<Page>,
}

impl LatestEarningsTranscriptsQuery {
    /// Creates a query without undocumented limit or page defaults.
    pub const fn new() -> Self {
        Self {
            limit: None,
            page: None,
        }
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the optional provider page index.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }

    /// Returns the optional provider page index.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }
}

impl QueryParameters for LatestEarningsTranscriptsQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("limit", self.limit);
        encoder.optional("page", self.page);
    }
}

/// Required transcript identity and optional provider result limit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EarningsTranscriptQuery {
    symbol: Ticker,
    year: Year,
    quarter: Quarter,
    limit: Option<Limit>,
}

impl EarningsTranscriptQuery {
    /// Creates a query for one ticker, year, and numeric query quarter.
    pub fn new(symbol: Ticker, year: Year, quarter: Quarter) -> Self {
        Self {
            symbol,
            year,
            quarter,
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

    /// Returns the requested query year.
    pub const fn year(&self) -> Year {
        self.year
    }

    /// Returns the requested textual query quarter.
    pub const fn quarter(&self) -> Quarter {
        self.quarter
    }

    /// Returns the optional provider result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }
}

impl QueryParameters for EarningsTranscriptQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.required("year", self.year);
        encoder.required("quarter", self.quarter);
        encoder.optional("limit", self.limit);
    }
}

/// Required ticker for available earnings-transcript dates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EarningsTranscriptDatesQuery {
    symbol: Ticker,
}

impl EarningsTranscriptDatesQuery {
    /// Creates a query for one ticker.
    pub const fn new(symbol: Ticker) -> Self {
        Self { symbol }
    }

    /// Borrows the requested ticker.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }
}

impl From<Ticker> for EarningsTranscriptDatesQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for EarningsTranscriptDatesQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for EarningsTranscriptDatesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
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
    fn latest_query_omits_absent_values_and_encodes_limit_then_page() {
        assert!(pairs(&LatestEarningsTranscriptsQuery::new()).is_empty());
        let query = LatestEarningsTranscriptsQuery::new()
            .with_limit(Limit(100))
            .with_page(Page(0));
        assert_eq!(
            pairs(&query),
            [
                ("limit".to_owned(), "100".to_owned()),
                ("page".to_owned(), "0".to_owned()),
            ]
        );
    }

    #[test]
    fn transcript_query_encodes_symbol_year_quarter_then_limit() {
        let query =
            EarningsTranscriptQuery::new(Ticker::new("AAPL").unwrap(), Year(2020), Quarter::Q3)
                .with_limit(Limit(1));
        assert_eq!(
            pairs(&query),
            [
                ("symbol".to_owned(), "AAPL".to_owned()),
                ("year".to_owned(), "2020".to_owned()),
                ("quarter".to_owned(), "3".to_owned()),
                ("limit".to_owned(), "1".to_owned()),
            ]
        );
    }

    #[test]
    fn transcript_dates_query_encodes_only_the_required_symbol() {
        let query = EarningsTranscriptDatesQuery::new(Ticker::new("AAPL").unwrap());
        assert_eq!(pairs(&query), [("symbol".to_owned(), "AAPL".to_owned())]);
    }
}
