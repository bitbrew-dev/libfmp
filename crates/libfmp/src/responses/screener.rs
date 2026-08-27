//! Response models returned by company screener endpoints.
//!
//! Future Python bindings should expose this model from `fmp.screener` as
//! `CompanyScreenerEntry`. This crate does not implement that Python binding.

use serde::{Deserialize, Serialize};

use crate::types::{
    CountryCode, ExchangeCode, Industry, MarketCapitalization, MarketValue, Price, Sector, Ticker,
    Volume,
};

/// One worldwide company returned by the stock screener.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyScreenerEntry {
    pub symbol: Ticker,
    pub company_name: String,
    pub market_cap: MarketCapitalization,
    pub sector: Sector,
    pub industry: Industry,
    pub beta: MarketValue,
    pub price: Price,
    pub last_annual_dividend: MarketValue,
    pub volume: Volume,
    /// The provider's full exchange name.
    pub exchange: String,
    /// The provider's abbreviated exchange code.
    pub exchange_short_name: ExchangeCode,
    pub country: CountryCode,
    pub is_etf: bool,
    pub is_fund: bool,
    pub is_actively_trading: bool,
}
