//! Response models returned by company profile, note, peer, delisting, workforce,
//! market-cap, share-float, merger-and-acquisition, and governance endpoints.
//!
//! The Python binding exposes these thirteen models under `fmp.company` with
//! the same names.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    codecs::{DynamicJson, NumericString, empty_or_null_date},
    types::{
        ApiDateTime, Change, Cik, Count, CountryCode, CurrencyCode, Cusip, Date, ExchangeCode,
        Industry, Isin, MarketCapitalization, MarketValue, PerShareAmount, Percentage, Price,
        Quantity, Ratio, Sector, Ticker, Volume,
    },
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// A detailed worldwide company profile returned by symbol or US CIK lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CompanyProfile {
    pub symbol: Ticker,
    pub price: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_cap: MarketCapitalization,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub beta: Ratio,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub last_dividend: PerShareAmount,
    pub range: String,
    pub change: Change,
    pub change_percentage: Percentage,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub volume: Volume,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub average_volume: Volume,
    pub company_name: String,
    pub currency: CurrencyCode,
    #[serde(deserialize_with = "required_option")]
    pub cik: Option<Cik>,
    pub isin: Isin,
    #[serde(deserialize_with = "required_option")]
    pub cusip: Option<Cusip>,
    pub exchange_full_name: String,
    pub exchange: ExchangeCode,
    pub industry: Industry,
    pub website: String,
    pub description: String,
    pub ceo: String,
    pub sector: Sector,
    pub country: CountryCode,
    #[serde(deserialize_with = "required_option")]
    pub full_time_employees: Option<NumericString>,
    #[serde(deserialize_with = "required_option")]
    pub phone: Option<String>,
    pub address: String,
    pub city: String,
    pub state: String,
    pub zip: String,
    pub image: String,
    #[serde(with = "empty_or_null_date")]
    pub ipo_date: Option<Date>,
    pub default_image: bool,
    pub is_etf: bool,
    pub is_actively_trading: bool,
    pub is_adr: bool,
    pub is_fund: bool,
}

/// One US company-issued note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CompanyNote {
    pub cik: Cik,
    pub symbol: Ticker,
    pub title: String,
    #[serde(deserialize_with = "required_option")]
    pub exchange: Option<ExchangeCode>,
}

/// One worldwide stock peer selected by the provider.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
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
#[non_exhaustive]
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
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EmployeeCount {
    pub symbol: Ticker,
    pub cik: Cik,
    pub acceptance_time: ApiDateTime,
    pub period_of_report: Date,
    pub company_name: String,
    pub form_type: String,
    pub filing_date: Date,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub employee_count: Count,
    pub source: String,
}

/// One current or historical worldwide market-capitalization observation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CompanyMarketCapitalization {
    pub symbol: Ticker,
    pub date: Date,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub market_cap: Option<MarketCapitalization>,
}

/// One worldwide company share-float observation with its filing source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CompanyShareFloat {
    pub symbol: Ticker,
    pub date: ApiDateTime,
    pub free_float: Percentage,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub float_shares: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub outstanding_shares: Quantity,
    #[serde(deserialize_with = "required_option")]
    pub source: Option<String>,
}

/// One all-company share-float observation, whose payload has no source field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct ShareFloat {
    pub symbol: Ticker,
    pub date: ApiDateTime,
    #[serde(deserialize_with = "required_option")]
    pub free_float: Option<Percentage>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub float_shares: Option<Quantity>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub outstanding_shares: Quantity,
}

/// One US merger or acquisition transaction and its official filing link.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct MergerAcquisition {
    pub symbol: Ticker,
    pub company_name: String,
    pub cik: Cik,
    pub targeted_company_name: String,
    #[serde(deserialize_with = "required_option")]
    pub targeted_cik: Option<Cik>,
    #[serde(deserialize_with = "required_option")]
    pub targeted_symbol: Option<Ticker>,
    pub transaction_date: Date,
    pub accepted_date: ApiDateTime,
    pub link: String,
}

/// One executive in a worldwide company's current leadership data.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CompanyExecutive {
    pub title: String,
    pub name: String,
    pub pay: Option<DynamicJson>,
    pub currency_pay: CurrencyCode,
    #[serde(deserialize_with = "required_option")]
    pub gender: Option<String>,
    pub year_born: Option<DynamicJson>,
    pub title_since: Option<DynamicJson>,
    pub active: bool,
}

/// One executive-compensation filing row for a US company.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
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
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub option_award: Option<MarketValue>,
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
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct ExecutiveCompensationBenchmark {
    pub industry_title: Industry,
    pub year: i64,
    pub average_compensation: f64,
}
