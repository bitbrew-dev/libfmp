//! Bulk end-of-day price rows.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::NumericString,
    types::{Date, Ticker},
};

/// One worldwide bulk end-of-day price row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkEodBar {
    pub symbol: Ticker,
    pub date: Date,
    pub open: NumericString,
    pub low: NumericString,
    pub high: NumericString,
    pub close: NumericString,
    pub adj_close: NumericString,
    pub volume: NumericString,
}
