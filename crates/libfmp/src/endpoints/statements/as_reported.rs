//! Query contracts and endpoint leaves for as-reported financial statements.

pub mod balance;
pub mod income;

pub use balance::balance_sheet_statement_as_reported;
pub use income::income_statement_as_reported;

use crate::{
    endpoints::{
        QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata},
    },
    query::RetrievalFrequency,
    types::{Limit, Ticker},
};

macro_rules! as_reported_query {
    ($docs:literal, $query:ident) => {
        #[doc = $docs]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $query {
            symbol: Ticker,
            limit: Option<Limit>,
            period: Option<RetrievalFrequency>,
        }

        impl $query {
            /// Creates a query for one ticker without undocumented defaults.
            pub fn new(symbol: Ticker) -> Self {
                Self {
                    symbol,
                    limit: None,
                    period: None,
                }
            }

            /// Sets the optional provider result limit.
            pub const fn with_limit(mut self, limit: Limit) -> Self {
                self.limit = Some(limit);
                self
            }

            /// Sets the optional annual or quarterly retrieval frequency.
            pub const fn with_period(mut self, period: RetrievalFrequency) -> Self {
                self.period = Some(period);
                self
            }

            /// Borrows the required ticker.
            pub fn symbol(&self) -> &Ticker {
                &self.symbol
            }

            /// Returns the optional provider result limit.
            pub const fn limit(&self) -> Option<Limit> {
                self.limit
            }

            /// Returns the optional annual or quarterly retrieval frequency.
            pub const fn period(&self) -> Option<RetrievalFrequency> {
                self.period
            }
        }

        impl From<Ticker> for $query {
            fn from(symbol: Ticker) -> Self {
                Self::new(symbol)
            }
        }

        impl From<&Ticker> for $query {
            fn from(symbol: &Ticker) -> Self {
                Self::new(symbol.clone())
            }
        }

        impl QueryParameters for $query {
            fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                encoder.required("symbol", &self.symbol);
                encoder.optional("limit", self.limit);
                encoder.optional("period", self.period);
            }
        }
    };
}

as_reported_query!(
    "Query parameters for income statements as reported by the company.",
    IncomeStatementAsReportedQuery
);
as_reported_query!(
    "Query parameters for balance sheets as reported by the company.",
    BalanceSheetStatementAsReportedQuery
);

pub(super) const AS_REPORTED_METADATA: EndpointMetadata =
    EndpointMetadata::new().with_bounds(EndpointBounds::new().with_response_rows(1_000));

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
    fn endpoint_owned_queries_preserve_omission_and_exact_parameter_order() {
        let symbol = Ticker::new("BRK.B / Class A").unwrap();
        let income: IncomeStatementAsReportedQuery = symbol.clone().into();
        let balance = BalanceSheetStatementAsReportedQuery::new(symbol.clone())
            .with_limit(Limit(1_000))
            .with_period(RetrievalFrequency::Quarterly);

        assert_eq!(income.symbol(), &symbol);
        assert_eq!(income.limit(), None);
        assert_eq!(income.period(), None);
        assert_eq!(
            pairs(&income),
            [("symbol".to_owned(), "BRK.B / Class A".to_owned())]
        );

        assert_eq!(balance.symbol(), &symbol);
        assert_eq!(balance.limit(), Some(Limit(1_000)));
        assert_eq!(balance.period(), Some(RetrievalFrequency::Quarterly));
        assert_eq!(
            pairs(&balance),
            [
                ("symbol".to_owned(), "BRK.B / Class A".to_owned()),
                ("limit".to_owned(), "1000".to_owned()),
                ("period".to_owned(), "quarter".to_owned()),
            ]
        );
    }

    #[test]
    fn both_endpoint_queries_encode_only_the_two_documented_frequencies() {
        let symbol = Ticker::new("AAPL").unwrap();
        for expected in ["annual", "quarter"] {
            let period = if expected == "annual" {
                RetrievalFrequency::Annual
            } else {
                RetrievalFrequency::Quarterly
            };
            assert_eq!(
                pairs(&IncomeStatementAsReportedQuery::new(symbol.clone()).with_period(period))[1]
                    .1,
                expected
            );
            assert_eq!(
                pairs(
                    &BalanceSheetStatementAsReportedQuery::new(symbol.clone()).with_period(period)
                )[1]
                .1,
                expected
            );
        }
    }
}
