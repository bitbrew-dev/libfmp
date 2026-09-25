//! Revenue-segmentation response models.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::DynamicObject,
    query::FiscalPeriod,
    types::{CalendarYear, CurrencyCode, Date, Ticker},
};

/// One product or geographic revenue-segmentation row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct RevenueSegmentation {
    pub symbol: Ticker,
    pub fiscal_year: CalendarYear,
    pub period: FiscalPeriod,
    pub reported_currency: CurrencyCode,
    pub date: Date,
    pub data: DynamicObject,
}
