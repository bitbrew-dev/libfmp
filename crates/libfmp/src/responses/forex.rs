//! Response models owned by forex endpoints.
//!
//! Detailed and compact forex quotes reuse the shared quote wire contracts.

use serde::{Deserialize, Serialize};

use crate::types::{CurrencyCode, Ticker};

pub use super::quote::{Quote, QuoteShort};

/// One currency pair in the provider's documented forex catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForexPair {
    pub symbol: Ticker,
    pub from_currency: CurrencyCode,
    pub to_currency: CurrencyCode,
    pub from_name: String,
    pub to_name: String,
}
