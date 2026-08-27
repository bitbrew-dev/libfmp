//! Response models returned by company profile, note, peer, delisting, workforce,
//! market-cap, share-float, and merger-and-acquisition endpoints.
//!
//! Future Python bindings should expose these models from `fmp.company` as
//! `CompanyProfile`, `CompanyNote`, `StockPeer`, `DelistedCompany`, and
//! `EmployeeCount`, plus `MarketCapitalizationRecord`, `CompanyShareFloat`, and
//! `AllSharesFloatRecord` and `MergerAcquisition`. This crate does not implement
//! those Python bindings.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::NumericString,
    types::{
        ApiDateTime, Change, Cik, Count, CountryCode, CurrencyCode, Cusip, Date, ExchangeCode,
        Industry, Isin, MarketCapitalization, MarketValue, Percentage, Price, Sector, Ticker,
        Volume,
    },
};

/// A detailed worldwide company profile returned by symbol or US CIK lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyProfile {
    pub symbol: Ticker,
    pub price: Price,
    pub market_cap: MarketCapitalization,
    pub beta: MarketValue,
    pub last_dividend: MarketValue,
    pub range: String,
    pub change: Change,
    pub change_percentage: Percentage,
    pub volume: Volume,
    pub average_volume: Volume,
    pub company_name: String,
    pub currency: CurrencyCode,
    pub cik: Cik,
    pub isin: Isin,
    pub cusip: Cusip,
    pub exchange_full_name: String,
    pub exchange: ExchangeCode,
    pub industry: Industry,
    pub website: String,
    pub description: String,
    pub ceo: String,
    pub sector: Sector,
    pub country: CountryCode,
    pub full_time_employees: NumericString,
    pub phone: String,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub image: String,
    pub ipo_date: Date,
    pub default_image: bool,
    pub is_etf: bool,
    pub is_actively_trading: bool,
    pub is_adr: bool,
    pub is_fund: bool,
}

/// One US company-issued note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyNote {
    pub cik: Cik,
    pub symbol: Ticker,
    pub title: String,
    pub exchange: ExchangeCode,
}

/// One worldwide stock peer selected by the provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockPeer {
    pub symbol: Ticker,
    pub company_name: String,
    pub price: Price,
    #[serde(rename = "mktCap")]
    pub market_cap: MarketCapitalization,
}

/// One US company that has been removed from an exchange.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelistedCompany {
    pub symbol: Ticker,
    pub company_name: String,
    pub exchange: ExchangeCode,
    pub ipo_date: Date,
    pub delisted_date: Date,
}

/// One current or historical US employee-count filing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmployeeCount {
    pub symbol: Ticker,
    pub cik: Cik,
    pub acceptance_time: ApiDateTime,
    pub period_of_report: Date,
    pub company_name: String,
    pub form_type: String,
    pub filing_date: Date,
    pub employee_count: Count,
    pub source: String,
}

/// One current or historical worldwide market-capitalization observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketCapitalizationRecord {
    pub symbol: Ticker,
    pub date: Date,
    pub market_cap: MarketCapitalization,
}

/// One worldwide company share-float observation with its filing source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyShareFloat {
    pub symbol: Ticker,
    pub date: ApiDateTime,
    pub free_float: Percentage,
    pub float_shares: Count,
    pub outstanding_shares: Count,
    pub source: String,
}

/// One all-company share-float observation, whose payload has no source field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllSharesFloatRecord {
    pub symbol: Ticker,
    pub date: ApiDateTime,
    pub free_float: Percentage,
    pub float_shares: Count,
    pub outstanding_shares: Count,
}

/// One US merger or acquisition transaction and its official filing link.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergerAcquisition {
    pub symbol: Ticker,
    pub company_name: String,
    pub cik: Cik,
    pub targeted_company_name: String,
    pub targeted_cik: Cik,
    pub targeted_symbol: Ticker,
    pub transaction_date: Date,
    pub accepted_date: ApiDateTime,
    pub link: String,
}
