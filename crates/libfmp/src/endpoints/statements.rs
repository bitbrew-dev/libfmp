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

pub mod as_reported;
pub mod balance;
pub mod cash_flow;
pub mod growth;
pub mod income;
pub mod metrics;
pub mod ratios;
pub mod reports;
pub mod segmentation;
pub mod summaries;

pub use as_reported::{
    BalanceSheetStatementAsReportedQuery, CashFlowStatementAsReportedQuery,
    FinancialStatementFullAsReportedQuery, IncomeStatementAsReportedQuery,
    balance_sheet_statement_as_reported, cash_flow_statement_as_reported,
    financial_statement_full_as_reported, income_statement_as_reported,
};
pub use balance::{balance_sheet_statement, balance_sheet_statement_ttm};
pub use cash_flow::{cash_flow_statement, cash_flow_statement_ttm};
pub use growth::{
    BalanceSheetStatementGrowthQuery, CashFlowStatementGrowthQuery, FinancialStatementGrowthQuery,
    IncomeStatementGrowthQuery, balance_sheet_statement_growth, cash_flow_statement_growth,
    financial_statement_growth, income_statement_growth,
};
pub use income::{income_statement, income_statement_ttm};
pub use metrics::{key_metrics, key_metrics_ttm};
pub use ratios::{financial_ratios, financial_ratios_ttm};
pub use reports::{financial_reports_dates, financial_reports_json, financial_reports_xlsx};
pub use segmentation::{
    RevenueGeographicSegmentationQuery, RevenueProductSegmentationQuery, SegmentationStructure,
    revenue_geographic_segmentation, revenue_product_segmentation,
};
pub use summaries::{
    enterprise_values, financial_scores, latest_financial_statements, owner_earnings,
};

use crate::{
    endpoints::{
        QueryEncoder, QueryParameters,
        metadata::{EndpointBounds, EndpointMetadata, GeographicAvailability},
    },
    query::StatementPeriod,
    types::{Limit, Page, Ticker},
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
statement_query!(
    "Query parameters for the worldwide historical key-metrics endpoint.",
    KeyMetricsQuery
);
statement_query!(
    "Query parameters for the worldwide financial-ratios endpoint.",
    FinancialRatiosQuery
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

/// Optional pagination for the worldwide latest-financial-statements endpoint.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LatestFinancialStatementsQuery {
    page: Option<Page>,
    limit: Option<Limit>,
}

impl LatestFinancialStatementsQuery {
    /// Creates a query without undocumented pagination defaults.
    pub const fn new() -> Self {
        Self {
            page: None,
            limit: None,
        }
    }

    /// Sets the optional provider page, including documented page zero.
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the optional provider result limit.
    pub const fn with_limit(mut self, limit: Limit) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Returns the optional page.
    pub const fn page(&self) -> Option<Page> {
        self.page
    }

    /// Returns the optional result limit.
    pub const fn limit(&self) -> Option<Limit> {
        self.limit
    }
}

impl QueryParameters for LatestFinancialStatementsQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.optional("page", self.page);
        encoder.optional("limit", self.limit);
    }
}

macro_rules! required_symbol_query {
    ($docs:literal, $query:ident) => {
        #[doc = $docs]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $query {
            symbol: Ticker,
        }

        impl $query {
            /// Creates a query for one ticker.
            pub fn new(symbol: Ticker) -> Self {
                Self { symbol }
            }

            /// Borrows the requested ticker.
            pub fn symbol(&self) -> &Ticker {
                &self.symbol
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
            }
        }
    };
}

required_symbol_query!(
    "Required query parameters for worldwide financial scores.",
    FinancialScoresQuery
);
required_symbol_query!(
    "Required query parameters for worldwide trailing-twelve-month key metrics.\n\nThe documented TTM query has no limit.\n\n```compile_fail\nuse libfmp::{endpoints::statements::KeyMetricsTtmQuery, types::{Limit, Ticker}};\nlet query = KeyMetricsTtmQuery::new(Ticker::new(\"AAPL\").unwrap());\nlet _ = query.with_limit(Limit(5));\n```\n\nThe documented TTM query has no period selector.\n\n```compile_fail\nuse libfmp::{endpoints::statements::KeyMetricsTtmQuery, query::FiscalPeriod, types::Ticker};\nlet query = KeyMetricsTtmQuery::new(Ticker::new(\"AAPL\").unwrap());\nlet _ = query.with_period(FiscalPeriod::Q1);\n```",
    KeyMetricsTtmQuery
);
required_symbol_query!(
    "Required query parameters for worldwide TTM financial ratios.\n\nTTM financial-ratio queries deliberately have neither limit nor period selectors.\n\n```compile_fail\nuse libfmp::{endpoints::statements::FinancialRatiosTtmQuery, types::{Limit, Ticker}};\nlet query = FinancialRatiosTtmQuery::new(Ticker::new(\"AAPL\").unwrap());\nlet _ = query.with_limit(Limit(5));\n```\n\n```compile_fail\nuse libfmp::{endpoints::statements::FinancialRatiosTtmQuery, query::FiscalPeriod, types::Ticker};\nlet query = FinancialRatiosTtmQuery::new(Ticker::new(\"AAPL\").unwrap());\nlet _ = query.with_period(FiscalPeriod::Q1);\n```",
    FinancialRatiosTtmQuery
);

macro_rules! symbol_limit_query {
    ($docs:literal, $query:ident) => {
        #[doc = $docs]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $query {
            symbol: Ticker,
            limit: Option<Limit>,
        }

        impl $query {
            /// Creates a query for one ticker without an undocumented limit.
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

symbol_limit_query!(
    "Query parameters for worldwide owner earnings.\n\nOwner-earnings queries deliberately have no period selector.\n\n```compile_fail\nuse libfmp::{endpoints::statements::OwnerEarningsQuery, query::FiscalPeriod, types::Ticker};\nlet query = OwnerEarningsQuery::new(Ticker::new(\"AAPL\").unwrap());\nlet _ = query.with_period(FiscalPeriod::Q1);\n```",
    OwnerEarningsQuery
);

/// Query parameters for worldwide enterprise values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnterpriseValuesQuery {
    symbol: Ticker,
    limit: Option<Limit>,
    period: Option<StatementPeriod>,
}

impl EnterpriseValuesQuery {
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

impl From<Ticker> for EnterpriseValuesQuery {
    fn from(symbol: Ticker) -> Self {
        Self::new(symbol)
    }
}

impl From<&Ticker> for EnterpriseValuesQuery {
    fn from(symbol: &Ticker) -> Self {
        Self::new(symbol.clone())
    }
}

impl QueryParameters for EnterpriseValuesQuery {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        encoder.required("symbol", &self.symbol);
        encoder.optional("limit", self.limit);
        encoder.optional("period", self.period);
    }
}

/// Shared documented metadata for worldwide statement endpoints with a
/// 1,000-row response cap.
pub(crate) const WORLDWIDE_STATEMENT_METADATA: EndpointMetadata = EndpointMetadata::new()
    .with_geography(GeographicAvailability::Worldwide)
    .with_bounds(EndpointBounds::new().with_response_rows(1_000));

/// Shared worldwide metadata for compact financial endpoints without bounds.
pub(crate) const WORLDWIDE_FINANCIAL_SUMMARY_METADATA: EndpointMetadata =
    EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);

/// Shared worldwide metadata for financial-history endpoints capped at 1,000 rows.
pub(crate) const WORLDWIDE_FINANCIAL_HISTORY_METADATA: EndpointMetadata = EndpointMetadata::new()
    .with_geography(GeographicAvailability::Worldwide)
    .with_bounds(EndpointBounds::new().with_response_rows(1_000));

/// Documented metadata for the latest-financial-statements endpoint.
pub(crate) const LATEST_FINANCIAL_STATEMENTS_METADATA: EndpointMetadata = EndpointMetadata::new()
    .with_geography(GeographicAvailability::Worldwide)
    .with_bounds(EndpointBounds::new().with_response_rows(250).with_page(100));

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

    #[test]
    fn compact_summary_queries_preserve_omission_and_documented_order() {
        let symbol = Ticker::new("BRK.B / Class A").unwrap();
        let latest = LatestFinancialStatementsQuery::new()
            .with_page(Page(0))
            .with_limit(Limit(250));
        let scores: FinancialScoresQuery = (&symbol).into();
        let owner = OwnerEarningsQuery::new(symbol.clone()).with_limit(Limit(5));
        let enterprise = EnterpriseValuesQuery::new(symbol.clone())
            .with_limit(Limit(1_000))
            .with_period(RetrievalFrequency::Quarterly);

        assert_eq!(latest.page(), Some(Page(0)));
        assert_eq!(latest.limit(), Some(Limit(250)));
        assert_eq!(scores.symbol(), &symbol);
        assert_eq!(owner.symbol(), &symbol);
        assert_eq!(owner.limit(), Some(Limit(5)));
        assert_eq!(enterprise.symbol(), &symbol);
        assert_eq!(enterprise.limit(), Some(Limit(1_000)));
        assert_eq!(
            enterprise.period(),
            Some(RetrievalFrequency::Quarterly.into())
        );
        assert_eq!(
            pairs(&latest),
            [
                ("page".to_owned(), "0".to_owned()),
                ("limit".to_owned(), "250".to_owned()),
            ]
        );
        assert_eq!(
            pairs(&scores),
            [("symbol".to_owned(), "BRK.B / Class A".to_owned())]
        );
        assert_eq!(
            pairs(&owner),
            [
                ("symbol".to_owned(), "BRK.B / Class A".to_owned()),
                ("limit".to_owned(), "5".to_owned()),
            ]
        );
        assert_eq!(
            pairs(&enterprise),
            [
                ("symbol".to_owned(), "BRK.B / Class A".to_owned()),
                ("limit".to_owned(), "1000".to_owned()),
                ("period".to_owned(), "quarter".to_owned()),
            ]
        );

        assert!(pairs(&LatestFinancialStatementsQuery::new()).is_empty());
        assert_eq!(
            pairs(&OwnerEarningsQuery::new(symbol.clone())),
            [("symbol".to_owned(), "BRK.B / Class A".to_owned())]
        );
        assert_eq!(
            pairs(&EnterpriseValuesQuery::new(symbol)),
            [("symbol".to_owned(), "BRK.B / Class A".to_owned())]
        );
    }

    #[test]
    fn compact_summary_metadata_records_only_documented_bounds() {
        assert_eq!(
            WORLDWIDE_FINANCIAL_SUMMARY_METADATA.geography(),
            GeographicAvailability::Worldwide
        );
        assert_eq!(
            WORLDWIDE_FINANCIAL_SUMMARY_METADATA.bounds(),
            EndpointBounds::new()
        );
        assert_eq!(
            WORLDWIDE_FINANCIAL_HISTORY_METADATA.bounds(),
            EndpointBounds::new().with_response_rows(1_000)
        );
        assert_eq!(
            LATEST_FINANCIAL_STATEMENTS_METADATA.geography(),
            GeographicAvailability::Worldwide
        );
        assert_eq!(
            LATEST_FINANCIAL_STATEMENTS_METADATA.bounds(),
            EndpointBounds::new().with_response_rows(250).with_page(100)
        );
        assert!(
            LATEST_FINANCIAL_STATEMENTS_METADATA
                .bounds()
                .accepts_page(Page(0))
        );
    }
}
