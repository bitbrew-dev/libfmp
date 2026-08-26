//! Response models returned by quote endpoints.

use serde::{Deserialize, Serialize};

use crate::types::{
    Change, ExchangeCode, MarketCapitalization, Percentage, Price, Ticker, UnixSeconds, Volume,
};

/// A detailed real-time stock quote.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub symbol: Ticker,
    pub name: String,
    pub price: Price,
    pub change_percentage: Percentage,
    pub change: Change,
    pub volume: Volume,
    pub day_low: Price,
    pub day_high: Price,
    pub year_high: Price,
    pub year_low: Price,
    pub market_cap: Option<MarketCapitalization>,
    pub price_avg_50: Price,
    pub price_avg_200: Price,
    pub exchange: ExchangeCode,
    pub open: Price,
    pub previous_close: Price,
    pub timestamp: UnixSeconds,
}

/// The compact response returned by quote-short endpoints across asset classes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteShort {
    pub symbol: Ticker,
    pub price: Price,
    pub change: Change,
    pub volume: Volume,
}
