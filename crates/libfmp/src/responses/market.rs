//! Response rows returned by market-performance endpoints.

use serde::{Deserialize, Serialize};

use crate::types::{Change, Date, ExchangeCode, Industry, Percentage, Price, Sector, Ticker};

/// One dated sector-level average market change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct SectorPerformance {
    pub date: Date,
    pub sector: Sector,
    pub exchange: ExchangeCode,
    pub average_change: Percentage,
}

/// One dated industry-level average market change.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct IndustryPerformance {
    pub date: Date,
    pub industry: Industry,
    pub exchange: ExchangeCode,
    pub average_change: Percentage,
}

/// One dated sector-level price-to-earnings ratio.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct SectorPe {
    pub date: Date,
    pub sector: Sector,
    pub exchange: ExchangeCode,
    pub pe: f64,
}

/// One dated industry-level price-to-earnings ratio.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct IndustryPe {
    pub date: Date,
    pub industry: Industry,
    pub exchange: ExchangeCode,
    pub pe: f64,
}

/// One stock in a provider-ranked market-mover list.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct MarketMover {
    pub symbol: Ticker,
    pub price: Price,
    pub name: String,
    pub change: Change,
    pub changes_percentage: Percentage,
    pub exchange: ExchangeCode,
}
