//! Response models owned by the worldwide market-hours endpoints.
//!
//! Opening hours, closing hours, and timezone identifiers preserve the
//! provider's raw strings. This module deliberately performs no timezone
//! parsing or conversion.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    codecs::DynamicJson,
    types::{Date, ExchangeCode},
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// Trading hours and current status for one exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeMarketHours {
    pub exchange: ExchangeCode,
    pub name: String,
    pub opening_hour: String,
    pub closing_hour: String,
    pub timezone: String,
    pub is_market_open: bool,
}

/// One holiday or non-trading day for an exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeHoliday {
    pub exchange: ExchangeCode,
    pub date: Date,
    pub name: String,
    pub is_closed: bool,
    #[serde(deserialize_with = "required_option")]
    pub adj_open_time: Option<DynamicJson>,
    #[serde(deserialize_with = "required_option")]
    pub adj_close_time: Option<DynamicJson>,
}
