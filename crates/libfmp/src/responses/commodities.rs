//! Response models owned by commodity endpoints.
//!
//! Detailed and compact commodity quotes reuse the shared quote wire contracts.

use serde::{Deserialize, Deserializer, Serialize};

use crate::types::{CurrencyCode, ExchangeCode, Ticker};

pub use super::quote::{Quote, QuoteShort};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One commodity in the provider's documented commodity catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommodityListing {
    pub symbol: Ticker,
    pub name: String,
    #[serde(deserialize_with = "required_option")]
    pub exchange: Option<ExchangeCode>,
    pub trade_month: String,
    pub currency: CurrencyCode,
}
