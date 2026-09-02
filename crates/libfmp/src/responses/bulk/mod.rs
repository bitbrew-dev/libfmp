//! Response rows returned by FMP bulk endpoints.
//!
//! Future Python bindings reserve these models under `fmp.bulk`.

mod snapshots;

pub use snapshots::{
    BulkDcfValuation, BulkEtfHolding, BulkFinancialScore, BulkPriceTargetSummary, BulkStockRating,
    BulkUpgradesDowngradesConsensus,
};
