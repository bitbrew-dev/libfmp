//! Response models returned by company profile, note, peer, delisting, workforce,
//! market-cap, share-float, merger-and-acquisition, and governance endpoints.
//!
//! The Python binding exposes these thirteen models under `fmp.company` with
//! the same names.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::{DynamicJson, NumericString},
    types::{
        ApiDateTime, Change, Cik, Count, CountryCode, CurrencyCode, Cusip, Date, ExchangeCode,
        Industry, Isin, MarketCapitalization, MarketValue, Percentage, Price, Quantity, Sector,
        Ticker, Volume,
    },
};

/// A detailed worldwide company profile returned by symbol or US CIK lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyProfile {
    pub symbol: Ticker,
    pub price: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_cap: MarketCapitalization,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub beta: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub last_dividend: MarketValue,
    pub range: String,
    pub change: Change,
    pub change_percentage: Percentage,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub volume: Volume,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
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
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketCapitalizationRecord {
    pub symbol: Ticker,
    pub date: Date,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_cap: MarketCapitalization,
}

/// One worldwide company share-float observation with its filing source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyShareFloat {
    pub symbol: Ticker,
    pub date: ApiDateTime,
    pub free_float: Percentage,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub float_shares: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub outstanding_shares: Quantity,
    pub source: String,
}

/// One all-company share-float observation, whose payload has no source field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllSharesFloatRecord {
    pub symbol: Ticker,
    pub date: ApiDateTime,
    pub free_float: Percentage,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub float_shares: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub outstanding_shares: Quantity,
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

/// One executive in a worldwide company's current leadership data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyExecutive {
    pub title: String,
    pub name: String,
    pub pay: Option<DynamicJson>,
    pub currency_pay: CurrencyCode,
    pub gender: String,
    pub year_born: Option<DynamicJson>,
    pub title_since: Option<DynamicJson>,
    pub active: bool,
}

/// One executive-compensation filing row for a US company.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutiveCompensation {
    pub cik: Cik,
    pub symbol: Ticker,
    pub company_name: String,
    pub filing_date: Date,
    pub accepted_date: ApiDateTime,
    pub name_and_position: String,
    pub year: i64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub salary: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub bonus: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub stock_award: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub option_award: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub incentive_plan_compensation: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub all_other_compensation: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total: MarketValue,
    pub link: String,
}

/// One US industry executive-compensation benchmark.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutiveCompensationBenchmark {
    pub industry_title: Industry,
    pub year: i64,
    pub average_compensation: f64,
}
