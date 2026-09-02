//! Response rows returned by FMP bulk endpoints.
//!
//! Future Python bindings reserve these models under `fmp.bulk`.

mod balance;
mod income;
mod metrics;
mod snapshots;

pub use balance::{BulkBalanceSheetStatement, BulkBalanceSheetStatementGrowth};
pub use income::{BulkIncomeStatement, BulkIncomeStatementGrowth};
pub use metrics::{
    BulkEarningsSurprise, BulkFinancialRatiosTtm, BulkKeyMetricsTtm, BulkStockPeers,
};
pub use snapshots::{
    BulkDcfValuation, BulkEtfHolding, BulkFinancialScore, BulkPriceTargetSummary, BulkStockRating,
    BulkUpgradesDowngradesConsensus,
};
