//! Response rows shared by stock chart endpoints.

use serde::{Deserialize, Serialize};

use crate::types::{ApiDateTime, Change, Date, Percentage, Price, Ticker, Volume};

/// One compact end-of-day stock chart row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct StockChartLightBar {
    pub symbol: Ticker,
    pub date: Date,
    pub price: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub volume: Volume,
}

/// One detailed end-of-day stock chart row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct StockChartFullBar {
    pub symbol: Ticker,
    pub date: Date,
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub volume: Volume,
    pub change: Change,
    pub change_percent: Percentage,
    pub vwap: Price,
}

/// One split-unadjusted or dividend-adjusted end-of-day stock chart row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct StockChartAdjustedBar {
    pub symbol: Ticker,
    pub date: Date,
    #[serde(rename = "adjOpen")]
    pub adj_open: Price,
    #[serde(rename = "adjHigh")]
    pub adj_high: Price,
    #[serde(rename = "adjLow")]
    pub adj_low: Price,
    #[serde(rename = "adjClose")]
    pub adj_close: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub volume: Volume,
}

/// One fixed-interval intraday stock chart row.
///
/// The provider row contains neither a ticker nor timezone information; the
/// timestamp is therefore represented by the strict timezone-less
/// [`ApiDateTime`] unit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct StockChartIntradayBar {
    pub date: ApiDateTime,
    pub open: Price,
    pub low: Price,
    pub high: Price,
    pub close: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub volume: Volume,
}
