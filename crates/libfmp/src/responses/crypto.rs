//! Response models owned by cryptocurrency endpoints.
//!
//! Detailed and compact cryptocurrency quotes reuse the shared quote wire contracts.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    codecs::empty_or_null_date,
    types::{Date, ExchangeCode, Ticker, TokenSupply},
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

pub use super::quote::{Quote, QuoteShort};

/// One cryptocurrency in the provider's documented cryptocurrency catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CryptocurrencyListing {
    pub symbol: Ticker,
    pub name: String,
    pub exchange: ExchangeCode,
    #[serde(with = "empty_or_null_date")]
    pub ico_date: Option<Date>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub circulating_supply: Option<TokenSupply>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub total_supply: Option<TokenSupply>,
}
