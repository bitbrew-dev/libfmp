//! Response models returned by symbol, transcript, and taxonomy directories.
//!
//! The Python binding exposes these models under `fmp.directory`.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    codecs::NumericString,
    types::{Cik, CountryCode, CurrencyCode, Date, ExchangeCode, Industry, Sector, Ticker},
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One worldwide company or instrument in the company-symbol directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CompanySymbol {
    pub symbol: Ticker,
    pub company_name: String,
}

/// One worldwide company with financial statements available from FMP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct FinancialStatementSymbol {
    pub symbol: Ticker,
    pub company_name: String,
    pub trading_currency: CurrencyCode,
    #[serde(deserialize_with = "crate::codecs::empty_or_null::deserialize")]
    pub reporting_currency: Option<CurrencyCode>,
}

/// One US SEC entity in the CIK directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CikListing {
    pub cik: Cik,
    pub company_name: String,
}

/// One US company symbol change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct SymbolChange {
    pub date: Date,
    pub company_name: String,
    pub old_symbol: Ticker,
    pub new_symbol: Ticker,
}

/// One worldwide exchange-traded fund in the ETF symbol directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EtfSymbol {
    pub symbol: Ticker,
    pub name: String,
}

/// One worldwide company or instrument in the actively-trading directory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct ActivelyTradingSymbol {
    pub symbol: Ticker,
    pub name: String,
}

/// One US company with the count of available earnings transcripts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EarningsTranscriptAvailability {
    pub symbol: Ticker,
    pub company_name: String,
    pub no_of_transcripts: NumericString,
}

/// One stock exchange supported by FMP.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct AvailableExchange {
    pub exchange: ExchangeCode,
    pub name: String,
    pub country_name: String,
    #[serde(deserialize_with = "crate::codecs::empty_or_null::deserialize")]
    pub country_code: Option<CountryCode>,
    pub symbol_suffix: String,
    #[serde(deserialize_with = "required_option")]
    pub delay: Option<String>,
}

/// One sector accepted by provider sector filters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AvailableSector {
    pub sector: Sector,
}

/// One industry accepted by provider industry filters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AvailableIndustry {
    pub industry: Industry,
}

/// One country accepted by provider country filters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct AvailableCountry {
    pub country: CountryCode,
}
