//! Response models owned by stock-market index endpoints.
//!
//! Generic detailed and compact quotes reuse [`super::quote::Quote`] and
//! [`super::quote::QuoteShort`] rather than duplicating their wire contracts.

use serde::{Deserialize, Serialize};

use crate::types::{CurrencyCode, ExchangeCode, Ticker};

/// One stock-market index in the provider's worldwide index directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexListing {
    pub symbol: Ticker,
    pub name: String,
    pub exchange: ExchangeCode,
    pub currency: CurrencyCode,
}
