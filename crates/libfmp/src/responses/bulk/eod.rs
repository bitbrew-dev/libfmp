//! Bulk end-of-day price rows.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::NumericString,
    types::{Date, Ticker},
};

/// One worldwide bulk end-of-day price row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkEodBar {
    pub symbol: Ticker,
    pub date: Date,
    pub open: Option<NumericString>,
    pub low: Option<NumericString>,
    pub high: Option<NumericString>,
    pub close: Option<NumericString>,
    pub adj_close: Option<NumericString>,
    pub volume: Option<NumericString>,
}
