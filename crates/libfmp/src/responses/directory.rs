//! Response models returned by symbol and transcript directory endpoints.
//!
//! Future Python bindings should expose the meaningful public models from
//! `fmp.directory` as `CompanySymbol`, `FinancialStatementSymbol`, `CikEntry`,
//! `SymbolChange`, `EtfSymbol`, `ActivelyTradingSymbol`, and
//! `EarningsTranscriptAvailability`. This crate does not implement those
//! Python bindings.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::NumericString,
    types::{Cik, CurrencyCode, Date, Ticker},
};

/// One worldwide company or instrument in the company-symbol directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanySymbol {
    pub symbol: Ticker,
    pub company_name: String,
}

/// One worldwide company with financial statements available from FMP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialStatementSymbol {
    pub symbol: Ticker,
    pub company_name: String,
    pub trading_currency: CurrencyCode,
    pub reporting_currency: CurrencyCode,
}

/// One US SEC entity in the CIK directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CikEntry {
    pub cik: Cik,
    pub company_name: String,
}

/// One US company symbol change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolChange {
    pub date: Date,
    pub company_name: String,
    pub old_symbol: Ticker,
    pub new_symbol: Ticker,
}

/// One worldwide exchange-traded fund in the ETF symbol directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EtfSymbol {
    pub symbol: Ticker,
    pub name: String,
}

/// One worldwide company or instrument in the actively-trading directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivelyTradingSymbol {
    pub symbol: Ticker,
    pub name: String,
}

/// One US company with the count of available earnings transcripts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EarningsTranscriptAvailability {
    pub symbol: Ticker,
    pub company_name: String,
    pub no_of_transcripts: NumericString,
}
