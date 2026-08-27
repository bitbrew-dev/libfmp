//! Query contracts shared by the standard and trailing-twelve-month financial
//! statement endpoints.
//!
//! Standard statement responses should reuse
//! [`crate::codecs::FiscalYearString`] for the documented string fiscal year
//! and [`crate::query::FiscalPeriod`] for response periods. Their request
//! queries use the wider [`crate::query::StatementPeriod`] union because the
//! provider also documents the `annual` and `quarter` retrieval frequencies.
//! Response rows and endpoint descriptors are intentionally defined by later
//! statement modules rather than this query foundation.

pub mod income;

pub use income::{income_statement, income_statement_ttm};

use crate::{
    endpoints::{
        QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata, GeographicAvailability},
    },
    query::StatementPeriod,
    types::{Limit, Ticker},
};

macro_rules! statement_query {
    ($docs:literal, $query:ident) => {
        #[doc = $docs]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $query {
            symbol: Ticker,
            limit: Option<Limit>,
            period: Option<StatementPeriod>,
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

            /// Sets the optional fiscal period or retrieval frequency.
            pub fn with_period(mut self, period: impl Into<StatementPeriod>) -> Self {
                self.period = Some(period.into());
                self
            }

            /// Borrows the requested ticker.
            pub fn symbol(&self) -> &Ticker {
                &self.symbol
            }

            /// Returns the optional provider result limit.
            pub const fn limit(&self) -> Option<Limit> {
                self.limit
            }

            /// Returns the optional fiscal period or retrieval frequency.
            pub const fn period(&self) -> Option<StatementPeriod> {
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

statement_query!(
    "Query parameters for the worldwide income-statement endpoint.",
    IncomeStatementQuery
);
statement_query!(
    "Query parameters for the worldwide balance-sheet-statement endpoint.",
    BalanceSheetStatementQuery
);
statement_query!(
    "Query parameters for the worldwide cash-flow-statement endpoint.",
    CashFlowStatementQuery
);

macro_rules! ttm_statement_query {
    ($docs:literal, $compile_fail:literal, $query:ident) => {
        #[doc = $docs]
        #[doc = $compile_fail]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $query {
            symbol: Ticker,
            limit: Option<Limit>,
        }

        impl $query {
            /// Creates a TTM query for one ticker without an undocumented limit.
            pub fn new(symbol: Ticker) -> Self {
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
            pub fn symbol(&self) -> &Ticker {
                &self.symbol
            }

            /// Returns the optional provider result limit.
            pub const fn limit(&self) -> Option<Limit> {
                self.limit
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
            }
        }
    };
}

ttm_statement_query!(
    "Query parameters for the worldwide income-statement TTM endpoint.",
    "\n\nTTM queries deliberately have no period selector.\n\n```compile_fail\nuse libfmp::{endpoints::statements::IncomeStatementTtmQuery, query::FiscalPeriod, types::Ticker};\nlet query = IncomeStatementTtmQuery::new(Ticker::new(\"AAPL\").unwrap());\nlet _ = query.with_period(FiscalPeriod::Q1);\n```",
    IncomeStatementTtmQuery
);
ttm_statement_query!(
    "Query parameters for the worldwide balance-sheet-statement TTM endpoint.",
    "\n\nTTM queries deliberately have no period selector.\n\n```compile_fail\nuse libfmp::{endpoints::statements::BalanceSheetStatementTtmQuery, query::FiscalPeriod, types::Ticker};\nlet query = BalanceSheetStatementTtmQuery::new(Ticker::new(\"AAPL\").unwrap());\nlet _ = query.with_period(FiscalPeriod::Q1);\n```",
    BalanceSheetStatementTtmQuery
);
ttm_statement_query!(
    "Query parameters for the worldwide cash-flow-statement TTM endpoint.",
    "\n\nTTM queries deliberately have no period selector.\n\n```compile_fail\nuse libfmp::{endpoints::statements::CashFlowStatementTtmQuery, query::FiscalPeriod, types::Ticker};\nlet query = CashFlowStatementTtmQuery::new(Ticker::new(\"AAPL\").unwrap());\nlet _ = query.with_period(FiscalPeriod::Q1);\n```",
    CashFlowStatementTtmQuery
);

/// Shared documented metadata for the six statement endpoints built on these
/// query contracts. Kept crate-private until endpoint descriptors consume it.
pub(crate) const WORLDWIDE_STATEMENT_METADATA: EndpointMetadata = EndpointMetadata::new()
    .with_geography(GeographicAvailability::Worldwide)
    .with_bounds(EndpointBounds::new().with_response_rows(1_000));

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::{FiscalPeriod, RetrievalFrequency};

    fn pairs(query: &impl QueryParameters) -> Vec<(String, String)> {
        let mut pairs = Vec::new();
        query.encode(&mut QueryEncoder::new(&mut |name, value| {
            pairs.push((name.to_owned(), value.to_owned()));
        }));
        pairs
    }

    #[test]
    fn standard_queries_omit_unspecified_values_and_preserve_endpoint_types() {
        let symbol = Ticker::new("AAPL").unwrap();
        let income: IncomeStatementQuery = symbol.clone().into();
        let balance: BalanceSheetStatementQuery = (&symbol).into();
        let cash = CashFlowStatementQuery::new(symbol.clone());

        for encoded in [pairs(&income), pairs(&balance), pairs(&cash)] {
            assert_eq!(encoded, [("symbol".to_owned(), "AAPL".to_owned())]);
        }
        assert_eq!(income.symbol(), &symbol);
        assert_eq!(income.limit(), None);
        assert_eq!(income.period(), None);
        assert_eq!(balance.symbol(), &symbol);
        assert_eq!(cash.symbol(), &symbol);
    }

    #[test]
    fn standard_query_builders_encode_symbol_limit_period_in_exact_order() {
        let symbol = Ticker::new("BRK.B / Class A").unwrap();
        let queries = [
            pairs(
                &IncomeStatementQuery::new(symbol.clone())
                    .with_limit(Limit(0))
                    .with_period(FiscalPeriod::Q1),
            ),
            pairs(
                &BalanceSheetStatementQuery::new(symbol.clone())
                    .with_limit(Limit(5))
                    .with_period(RetrievalFrequency::Annual),
            ),
            pairs(
                &CashFlowStatementQuery::new(symbol.clone())
                    .with_limit(Limit(1_000))
                    .with_period(FiscalPeriod::FullYear),
            ),
        ];

        assert_eq!(
            queries[0],
            [
                ("symbol".to_owned(), "BRK.B / Class A".to_owned()),
                ("limit".to_owned(), "0".to_owned()),
                ("period".to_owned(), "Q1".to_owned()),
            ]
        );
        assert_eq!(
            queries[1][1..],
            [
                ("limit".to_owned(), "5".to_owned()),
                ("period".to_owned(), "annual".to_owned())
            ]
        );
        assert_eq!(
            queries[2][1..],
            [
                ("limit".to_owned(), "1000".to_owned()),
                ("period".to_owned(), "FY".to_owned())
            ]
        );
    }

    #[test]
    fn all_seven_statement_period_spellings_remain_distinct() {
        let periods = [
            StatementPeriod::from(FiscalPeriod::Q1),
            StatementPeriod::from(FiscalPeriod::Q2),
            StatementPeriod::from(FiscalPeriod::Q3),
            StatementPeriod::from(FiscalPeriod::Q4),
            StatementPeriod::from(FiscalPeriod::FullYear),
            StatementPeriod::from(RetrievalFrequency::Annual),
            StatementPeriod::from(RetrievalFrequency::Quarterly),
        ];
        let symbol = Ticker::new("AAPL").unwrap();

        assert_eq!(
            periods.map(|period| {
                pairs(&IncomeStatementQuery::new(symbol.clone()).with_period(period))[1]
                    .1
                    .clone()
            }),
            ["Q1", "Q2", "Q3", "Q4", "FY", "annual", "quarter"]
        );
    }

    #[test]
    fn ttm_queries_have_only_symbol_and_optional_limit() {
        let symbol = Ticker::new("AAPL").unwrap();
        let income: IncomeStatementTtmQuery = symbol.clone().into();
        let balance: BalanceSheetStatementTtmQuery = (&symbol).into();
        let cash = CashFlowStatementTtmQuery::new(symbol.clone());

        for encoded in [pairs(&income), pairs(&balance), pairs(&cash)] {
            assert_eq!(encoded, [("symbol".to_owned(), "AAPL".to_owned())]);
        }
        assert_eq!(income.symbol(), &symbol);
        assert_eq!(income.limit(), None);
        assert_eq!(balance.symbol(), &symbol);
        assert_eq!(cash.symbol(), &symbol);

        for encoded in [
            pairs(&income.with_limit(Limit(0))),
            pairs(&balance.with_limit(Limit(5))),
            pairs(&cash.with_limit(Limit(1_000))),
        ] {
            assert_eq!(encoded[0], ("symbol".to_owned(), "AAPL".to_owned()));
            assert_eq!(encoded[1].0, "limit");
            assert_eq!(encoded.len(), 2);
        }
    }

    #[test]
    fn shared_metadata_records_worldwide_coverage_and_response_cap() {
        assert_eq!(
            WORLDWIDE_STATEMENT_METADATA.geography(),
            GeographicAvailability::Worldwide
        );
        assert_eq!(
            WORLDWIDE_STATEMENT_METADATA
                .bounds()
                .response_rows()
                .unwrap()
                .maximum(),
            1_000
        );
    }
}
