//! Analyst endpoint query contracts.

use crate::{
    endpoints::{QueryEncoder, QueryParameters},
    query::RetrievalFrequency,
    types::{Limit, Page, Ticker},
};

/// Required security and frequency plus optional pagination for financial estimates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinancialEstimatesQuery {
    symbol: Ticker,
    period: RetrievalFrequency,
    page: Option<Page>,
    limit: Option<Limit>,
}

impl FinancialEstimatesQuery {
    /// Creates a query without undocumented pagination defaults.
    pub const fn new(symbol: Ticker, period: RetrievalFrequency) -> Self {
        Self {
            symbol,
            period,
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

    /// Borrows the requested ticker.
    pub const fn symbol(&self) -> &Ticker {
        &self.symbol
    }

    /// Returns the required annual or quarterly retrieval frequency.
    pub const fn period(&self) -> RetrievalFrequency {
        self.period
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

impl QueryParameters for FinancialEstimatesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.required("period", self.period);
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

macro_rules! required_symbol_query {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            symbol: Ticker,
        }

        impl $name {
            /// Creates a query for one ticker.
            pub const fn new(symbol: Ticker) -> Self {
                Self { symbol }
            }

            /// Borrows the requested ticker.
            pub const fn symbol(&self) -> &Ticker {
                &self.symbol
            }
        }

        impl From<Ticker> for $name {
            fn from(symbol: Ticker) -> Self {
                Self::new(symbol)
            }
        }

        impl From<&Ticker> for $name {
            fn from(symbol: &Ticker) -> Self {
                Self::new(symbol.clone())
            }
        }

        impl QueryParameters for $name {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required("symbol", &self.symbol);
            }
        }
    };
}

macro_rules! symbol_limit_query {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            symbol: Ticker,
            limit: Option<Limit>,
        }

        impl $name {
            /// Creates a query for one ticker without an undocumented limit.
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

        impl From<Ticker> for $name {
            fn from(symbol: Ticker) -> Self {
                Self::new(symbol)
            }
        }

        impl From<&Ticker> for $name {
            fn from(symbol: &Ticker) -> Self {
                Self::new(symbol.clone())
            }
        }

        impl QueryParameters for $name {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required("symbol", &self.symbol);
                encoder.optional("limit", self.limit);
            }
        }
    };
}

required_symbol_query!(
    RatingsSnapshotQuery,
    "Required ticker for the worldwide ratings snapshot."
);
symbol_limit_query!(
    HistoricalRatingsQuery,
    "Required ticker and optional limit for historical worldwide ratings."
);
required_symbol_query!(
    PriceTargetSummaryQuery,
    "Required ticker for the US price-target summary."
);
required_symbol_query!(
    PriceTargetConsensusQuery,
    "Required ticker for the US price-target consensus."
);
required_symbol_query!(
    StockGradesQuery,
    "Required ticker for current worldwide stock grades."
);
symbol_limit_query!(
    HistoricalStockGradesQuery,
    "Required ticker and optional limit for historical worldwide stock grades."
);
required_symbol_query!(
    StockGradesSummaryQuery,
    "Required ticker for the worldwide stock-grades summary."
);

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
    fn estimates_encode_exact_keys_and_order() {
        let query = FinancialEstimatesQuery::new(
            Ticker::new("BRK.B / Class A").unwrap(),
            RetrievalFrequency::Quarterly,
        )
        .with_page(Page(0))
        .with_limit(Limit(u32::MAX));
        assert_eq!(
            pairs(&query),
            [
                ("symbol".to_owned(), "BRK.B / Class A".to_owned()),
                ("period".to_owned(), "quarter".to_owned()),
                ("page".to_owned(), "0".to_owned()),
                ("limit".to_owned(), u32::MAX.to_string()),
            ]
        );
    }

    #[test]
    fn symbol_only_and_symbol_limit_queries_encode_exact_shapes() {
        let symbol = Ticker::new("AAPL").unwrap();
        assert_eq!(
            pairs(&RatingsSnapshotQuery::new(symbol.clone())),
            [("symbol".to_owned(), "AAPL".to_owned())]
        );
        assert_eq!(
            pairs(&HistoricalRatingsQuery::new(symbol.clone())),
            [("symbol".to_owned(), "AAPL".to_owned())]
        );
        assert_eq!(
            pairs(&HistoricalRatingsQuery::new(symbol.clone()).with_limit(Limit(0))),
            [
                ("symbol".to_owned(), "AAPL".to_owned()),
                ("limit".to_owned(), "0".to_owned()),
            ]
        );
        assert_eq!(
            pairs(&HistoricalStockGradesQuery::new(symbol).with_limit(Limit(u32::MAX))),
            [
                ("symbol".to_owned(), "AAPL".to_owned()),
                ("limit".to_owned(), u32::MAX.to_string()),
            ]
        );
    }
}
