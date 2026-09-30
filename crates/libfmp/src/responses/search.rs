//! Response models returned by search endpoints.
//!
//! The Python binding exposes these models under `fmp.search`.

use serde::{Deserialize, Deserializer, Serialize};

use crate::types::{
    Change, Cik, CountryCode, CurrencyCode, Cusip, Date, ExchangeCode, Isin, MarketCapitalization,
    PerShareAmount, Price, Ratio, Ticker, Volume,
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// A company or instrument returned by symbol search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct SymbolSearchResult {
    pub symbol: Ticker,
    pub name: String,
    pub currency: CurrencyCode,
    pub exchange_full_name: String,
    pub exchange: ExchangeCode,
}

/// A company or instrument returned by name search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct NameSearchResult {
    pub symbol: Ticker,
    pub name: String,
    pub currency: CurrencyCode,
    pub exchange_full_name: String,
    pub exchange: ExchangeCode,
}

/// A US company returned by CIK search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CikSearchResult {
    pub symbol: Ticker,
    pub company_name: String,
    pub cik: Cik,
    pub exchange_full_name: String,
    pub exchange: ExchangeCode,
    pub currency: CurrencyCode,
}

/// A security returned by CUSIP search.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CusipSearchResult {
    pub symbol: Ticker,
    pub company_name: String,
    pub cusip: Cusip,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_cap: MarketCapitalization,
}

/// A security returned by ISIN search.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct IsinSearchResult {
    pub symbol: Ticker,
    pub name: String,
    pub isin: Isin,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_cap: MarketCapitalization,
}

/// One exchange listing returned by exchange-variants search.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct ExchangeVariant {
    pub symbol: Ticker,
    #[serde(deserialize_with = "required_option")]
    pub price: Option<Price>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub beta: Ratio,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub vol_avg: Volume,
    #[serde(rename = "mktCap")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_cap: MarketCapitalization,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub last_div: PerShareAmount,
    #[serde(deserialize_with = "required_option")]
    pub range: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub changes: Option<Change>,
    pub company_name: String,
    pub currency: CurrencyCode,
    #[serde(deserialize_with = "required_option")]
    pub cik: Option<Cik>,
    #[serde(deserialize_with = "required_option")]
    pub isin: Option<Isin>,
    #[serde(deserialize_with = "crate::codecs::empty_or_null::deserialize")]
    pub cusip: Option<Cusip>,
    /// The provider's `exchange` field is the full exchange name here.
    pub exchange: String,
    /// The provider's `exchangeShortName` field is the exchange code here.
    pub exchange_short_name: ExchangeCode,
    pub industry: String,
    #[serde(deserialize_with = "required_option")]
    pub website: Option<String>,
    pub description: String,
    #[serde(deserialize_with = "required_option")]
    pub ceo: Option<String>,
    pub sector: String,
    pub country: CountryCode,
    #[serde(deserialize_with = "required_option")]
    pub full_time_employees: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub phone: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub address: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub city: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub state: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub zip: Option<String>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub dcf_diff: Option<PerShareAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub dcf: PerShareAmount,
    pub image: String,
    pub ipo_date: Date,
    pub default_image: bool,
    pub is_etf: bool,
    pub is_actively_trading: bool,
    pub is_adr: bool,
    pub is_fund: bool,
}
