//! Response rows returned by earnings-transcript endpoints.

pub use crate::responses::directory::EarningsTranscriptAvailability;

use serde::{Deserialize, Serialize};

use crate::{
    query::FiscalPeriod,
    types::{CalendarQuarter, CalendarYear, Date, Ticker},
};

/// One latest worldwide earnings-transcript metadata row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestEarningsTranscript {
    pub symbol: Ticker,
    pub period: FiscalPeriod,
    pub fiscal_year: CalendarYear,
    pub date: Date,
}

/// One complete worldwide earnings-call transcript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsTranscript {
    pub symbol: Ticker,
    pub period: FiscalPeriod,
    pub year: CalendarYear,
    pub date: Date,
    pub content: String,
}

/// One available transcript date and its numeric fiscal quarter/year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsTranscriptDate {
    pub quarter: CalendarQuarter,
    pub fiscal_year: CalendarYear,
    pub date: Date,
}
