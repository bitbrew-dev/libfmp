//! Response models returned by company profile, note, and peer endpoints.
//!
//! Future Python bindings should expose these models from `fmp.company` as
//! `CompanyProfile`, `CompanyNote`, and `StockPeer`. This crate does not
//! implement those Python bindings.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::NumericString,
    types::{
        Change, Cik, CountryCode, CurrencyCode, Cusip, Date, ExchangeCode, Industry, Isin,
        MarketCapitalization, MarketValue, Percentage, Price, Sector, Ticker, Volume,
    },
};

/// A detailed worldwide company profile returned by symbol or US CIK lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyProfile {
    pub symbol: Ticker,
    pub price: Price,
    pub market_cap: MarketCapitalization,
    pub beta: MarketValue,
    pub last_dividend: MarketValue,
    pub range: String,
    pub change: Change,
    pub change_percentage: Percentage,
    pub volume: Volume,
    pub average_volume: Volume,
    pub company_name: String,
    pub currency: CurrencyCode,
    pub cik: Cik,
    pub isin: Isin,
    pub cusip: Cusip,
    pub exchange_full_name: String,
    pub exchange: ExchangeCode,
    pub industry: Industry,
    pub website: String,
    pub description: String,
    pub ceo: String,
    pub sector: Sector,
    pub country: CountryCode,
    pub full_time_employees: NumericString,
    pub phone: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub image: String,
    pub ipo_date: Date,
    pub default_image: bool,
    pub is_etf: bool,
    pub is_actively_trading: bool,
    pub is_adr: bool,
    pub is_fund: bool,
}

/// One US company-issued note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyNote {
    pub cik: Cik,
    pub symbol: Ticker,
    pub title: String,
    pub exchange: ExchangeCode,
}

/// One worldwide stock peer selected by the provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockPeer {
    pub symbol: Ticker,
    pub company_name: String,
    pub price: Price,
    #[serde(rename = "mktCap")]
    pub market_cap: MarketCapitalization,
}
