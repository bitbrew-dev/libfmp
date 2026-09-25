//! Response models owned by cryptocurrency endpoints.
//!
//! Detailed and compact cryptocurrency quotes reuse the shared quote wire contracts.

use serde::{Deserialize, Serialize};

use crate::types::{Date, ExchangeCode, Ticker, TokenSupply};

pub use super::quote::{Quote, QuoteShort};

/// One cryptocurrency in the provider's documented cryptocurrency catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CryptocurrencyListing {
    pub symbol: Ticker,
    pub name: String,
    pub exchange: ExchangeCode,
    pub ico_date: Date,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub circulating_supply: TokenSupply,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_supply: TokenSupply,
}
