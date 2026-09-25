//! Response models returned by search endpoints.
//!
//! The Python binding exposes these models under `fmp.search`.

use serde::{Deserialize, Serialize};

use crate::types::{
    Change, Cik, CountryCode, CurrencyCode, Cusip, Date, ExchangeCode, Isin, MarketCapitalization,
    MarketValue, Price, Ticker, Volume,
};

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
    pub price: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub beta: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub vol_avg: Volume,
    #[serde(rename = "mktCap")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_cap: MarketCapitalization,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub last_div: MarketValue,
    pub range: String,
    pub changes: Change,
    pub company_name: String,
    pub currency: CurrencyCode,
    pub cik: Cik,
    pub isin: Isin,
    pub cusip: Cusip,
    /// The provider's `exchange` field is the full exchange name here.
    pub exchange: String,
    /// The provider's `exchangeShortName` field is the exchange code here.
    pub exchange_short_name: ExchangeCode,
    pub industry: String,
    pub website: String,
    pub description: String,
    pub ceo: String,
    pub sector: String,
    pub country: CountryCode,
    pub full_time_employees: String,
    pub phone: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub dcf_diff: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub dcf: MarketValue,
    pub image: String,
    pub ipo_date: Date,
    pub default_image: bool,
    pub is_etf: bool,
    pub is_actively_trading: bool,
    pub is_adr: bool,
    pub is_fund: bool,
}
