//! Response rows returned by FMP bulk endpoints.
//!
//! Provider numeric strings are kept verbatim as
//! [`crate::codecs::NumericString`], so very large or high-precision values
//! never pass through a float. Documented wire keys that look malformed are
//! mirrored as explicit serde renames rather than corrected: `"Stock Price"`,
//! `lastUpdated"`, the `growthOthertotalStockholdersEquity` casing, and the
//! `Activites` spellings on cash-flow growth rows. The Python binding exposes
//! these models under `fmp.bulk`.

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
