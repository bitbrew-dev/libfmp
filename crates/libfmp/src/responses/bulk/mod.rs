//! Response rows returned by FMP bulk endpoints.
//!
//! Future Python bindings reserve these models under `fmp.bulk`.

mod balance;
mod cash_flow;
mod eod;
mod income;
mod metrics;
mod snapshots;

pub use balance::{BulkBalanceSheetStatement, BulkBalanceSheetStatementGrowth};
pub use cash_flow::{BulkCashFlowStatement, BulkCashFlowStatementGrowth};
pub use eod::BulkEodBar;
pub use income::{BulkIncomeStatement, BulkIncomeStatementGrowth};
pub use metrics::{
    BulkEarningsSurprise, BulkFinancialRatiosTtm, BulkKeyMetricsTtm, BulkStockPeers,
};
pub use snapshots::{
    BulkDcfValuation, BulkEtfHolding, BulkFinancialScore, BulkPriceTargetSummary, BulkStockRating,
    BulkUpgradesDowngradesConsensus,
};
