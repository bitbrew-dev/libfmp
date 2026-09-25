//! Shared envelope for financial statements as reported by a company.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::DynamicObject,
    query::FiscalPeriod,
    types::{CalendarYear, CurrencyCode, Date, Ticker},
};

/// One as-reported financial statement with provider-native data keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct AsReportedFinancialStatement {
    pub symbol: Ticker,
    pub fiscal_year: CalendarYear,
    pub period: FiscalPeriod,
    pub reported_currency: CurrencyCode,
    pub date: Date,
    pub data: DynamicObject,
}
