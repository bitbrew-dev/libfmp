//! Response models owned by stock-market index endpoints.
//!
//! Generic detailed and compact quotes reuse [`super::quote::Quote`] and
//! [`super::quote::QuoteShort`] rather than duplicating their wire contracts.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    codecs::OpaqueDateText,
    types::{Cik, CurrencyCode, Date, ExchangeCode, Industry, Sector, Ticker},
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One stock-market index in the provider's worldwide index directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct IndexListing {
    pub symbol: Ticker,
    pub name: String,
    pub exchange: ExchangeCode,
    pub currency: CurrencyCode,
}

/// One company currently included in a major US stock-market index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct IndexConstituent {
    pub symbol: Ticker,
    pub name: String,
    pub sector: Sector,
    pub sub_sector: Industry,
    pub head_quarter: String,
    #[serde(deserialize_with = "required_option")]
    pub date_first_added: Option<Date>,
    pub cik: Cik,
    pub founded: Date,
}

/// One documented historical addition to and removal from a major US index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct HistoricalIndexConstituent {
    pub date_added: OpaqueDateText,
    pub added_security: String,
    #[serde(deserialize_with = "required_option")]
    pub removed_ticker: Option<Ticker>,
    #[serde(deserialize_with = "required_option")]
    pub removed_security: Option<String>,
    pub date: Date,
    pub symbol: Ticker,
    pub reason: String,
}
