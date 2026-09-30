//! Response models returned by company screener endpoints.
//!
//! The Python binding exposes this model under `fmp.screener`.

use serde::{Deserialize, Deserializer, Serialize};

use crate::types::{
    CountryCode, ExchangeCode, Industry, MarketCapitalization, PerShareAmount, Price, Ratio,
    Sector, Ticker, Volume,
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One worldwide company returned by the stock screener.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CompanyScreenerResult {
    pub symbol: Ticker,
    pub company_name: String,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_cap: MarketCapitalization,
    pub sector: Sector,
    pub industry: Industry,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub beta: Option<Ratio>,
    #[serde(deserialize_with = "required_option")]
    pub price: Option<Price>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub last_annual_dividend: Option<PerShareAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub volume: Volume,
    /// The provider's full exchange name.
    pub exchange: String,
    /// The provider's abbreviated exchange code.
    pub exchange_short_name: ExchangeCode,
    #[serde(deserialize_with = "required_option")]
    pub country: Option<CountryCode>,
    pub is_etf: bool,
    #[serde(deserialize_with = "required_option")]
    pub is_fund: Option<bool>,
    pub is_actively_trading: bool,
}
