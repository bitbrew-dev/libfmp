//! Response models returned by quote endpoints.

use serde::{Deserialize, Serialize};

use crate::types::{
    Change, ExchangeCode, MarketCapitalization, Percentage, Price, Quantity, Ticker,
    UnixMilliseconds, UnixSeconds, Volume,
};

/// A detailed real-time stock quote.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    pub symbol: Ticker,
    pub name: String,
    pub price: Price,
    pub change_percentage: Percentage,
    pub change: Change,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub volume: Volume,
    pub day_low: Price,
    pub day_high: Price,
    pub year_high: Price,
    pub year_low: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
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
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct QuoteShort {
    pub symbol: Ticker,
    pub price: Price,
    pub change: Change,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub volume: Volume,
}

/// One trade executed after regular US market hours.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct AftermarketTrade {
    pub symbol: Ticker,
    pub price: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub trade_size: Quantity,
    pub timestamp: UnixMilliseconds,
}

/// One bid-and-ask quote observed after regular US market hours.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct AftermarketQuote {
    pub symbol: Ticker,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub bid_size: Quantity,
    pub bid_price: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub ask_size: Quantity,
    pub ask_price: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub volume: Volume,
    pub timestamp: UnixMilliseconds,
}

/// Percentage changes for one stock across the provider's documented periods.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StockPriceChange {
    pub symbol: Ticker,
    #[serde(rename = "1D")]
    pub one_day: Percentage,
    #[serde(rename = "5D")]
    pub five_days: Percentage,
    #[serde(rename = "1M")]
    pub one_month: Percentage,
    #[serde(rename = "3M")]
    pub three_months: Percentage,
    #[serde(rename = "6M")]
    pub six_months: Percentage,
    #[serde(rename = "ytd")]
    pub year_to_date: Percentage,
    #[serde(rename = "1Y")]
    pub one_year: Percentage,
    #[serde(rename = "3Y")]
    pub three_years: Percentage,
    #[serde(rename = "5Y")]
    pub five_years: Percentage,
    #[serde(rename = "10Y")]
    pub ten_years: Percentage,
    pub max: Percentage,
}
