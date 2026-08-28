//! Query contracts and endpoint leaves for financial-statement growth.

pub mod balance;
pub mod income;

pub use balance::balance_sheet_statement_growth;
pub use income::income_statement_growth;

use crate::{
    endpoints::{QueryEncoder, QueryParameters},
    query::StatementPeriod,
    types::{Limit, Ticker},
};

macro_rules! growth_query {
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

growth_query!(
    "Query parameters for worldwide income-statement growth.",
    IncomeStatementGrowthQuery
);
growth_query!(
    "Query parameters for worldwide balance-sheet-statement growth.",
    BalanceSheetStatementGrowthQuery
);

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
    fn endpoint_specific_queries_preserve_omission_accessors_and_exact_order() {
        let symbol = Ticker::new("BRK.B / Class A").unwrap();
        let income: IncomeStatementGrowthQuery = symbol.clone().into();
        let balance = BalanceSheetStatementGrowthQuery::new(symbol.clone())
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
        assert_eq!(balance.period(), Some(RetrievalFrequency::Quarterly.into()));
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
    fn both_growth_queries_preserve_all_seven_statement_period_spellings() {
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

        for encoded in [
            periods.map(|period| {
                pairs(&IncomeStatementGrowthQuery::new(symbol.clone()).with_period(period))[1]
                    .1
                    .clone()
            }),
            periods.map(|period| {
                pairs(&BalanceSheetStatementGrowthQuery::new(symbol.clone()).with_period(period))[1]
                    .1
                    .clone()
            }),
        ] {
            assert_eq!(encoded, ["Q1", "Q2", "Q3", "Q4", "FY", "annual", "quarter"]);
        }
    }
}
