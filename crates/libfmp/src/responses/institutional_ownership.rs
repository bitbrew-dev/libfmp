//! Response rows returned by institutional-ownership filing endpoints.

use serde::{Deserialize, Serialize};

use crate::types::{ApiDateTime, CalendarQuarter, CalendarYear, Cik, Cusip, Date, Ticker};

/// One recent institutional-ownership filing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionalOwnershipFiling {
    pub cik: Cik,
    pub name: String,
    pub date: Date,
    pub filing_date: ApiDateTime,
    pub accepted_date: ApiDateTime,
    pub form_type: String,
    pub link: String,
    pub final_link: String,
}

/// One security position extracted from an institutional-ownership filing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionalHolding {
    pub date: Date,
    pub filing_date: Date,
    pub accepted_date: Date,
    pub cik: Cik,
    pub security_cusip: Cusip,
    pub symbol: Ticker,
    pub name_of_issuer: String,
    pub shares: u64,
    pub title_of_class: String,
    pub shares_type: String,
    pub put_call_share: String,
    pub value: u64,
    pub link: String,
    pub final_link: String,
}

/// One available Form 13F reporting date and its numeric calendar period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Form13fFilingDate {
    pub date: Date,
    pub year: CalendarYear,
    pub quarter: CalendarQuarter,
}
