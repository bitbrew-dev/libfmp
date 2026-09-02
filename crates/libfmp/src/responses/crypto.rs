//! Response models owned by cryptocurrency endpoints.
//!
//! Detailed and compact cryptocurrency quotes reuse the shared quote wire contracts.

use serde::{Deserialize, Serialize};

use crate::types::{Date, ExchangeCode, Ticker};

pub use super::quote::{Quote, QuoteShort};

/// One cryptocurrency in the provider's documented cryptocurrency catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CryptocurrencyListing {
    pub symbol: Ticker,
    pub name: String,
    pub exchange: ExchangeCode,
    pub ico_date: Date,
    pub circulating_supply: u64,
    pub total_supply: u64,
}
