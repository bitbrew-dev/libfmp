//! Response models owned by discounted-cash-flow valuation endpoints.

use serde::{Deserialize, Serialize};

use crate::types::{Date, Ticker};

/// One standard or levered discounted-cash-flow valuation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DcfValuation {
    pub symbol: Ticker,
    pub date: Date,
    pub dcf: f64,
    #[serde(rename = "Stock Price")]
    pub stock_price: f64,
}
