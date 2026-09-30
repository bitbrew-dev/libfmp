//! Response rows returned by insider-trading endpoints.
//!
//! The provider documents every insider-trading route as available for
//! US-based companies only. The Python binding exposes these models under
//! `fmp.insider_trading`.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    codecs::NumericString,
    types::{
        CalendarQuarter, CalendarYear, Cik, Count, Cusip, Date, FormType, Price, Quantity, Ticker,
        TransactionTypeCode,
    },
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One insider trade shared by the latest and search feeds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct InsiderTrade {
    pub symbol: Ticker,
    pub filing_date: Date,
    pub transaction_date: Date,
    pub reporting_cik: Cik,
    pub company_cik: Cik,
    #[serde(deserialize_with = "crate::codecs::empty_or_null::deserialize")]
    pub transaction_type: Option<TransactionTypeCode>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub securities_owned: Option<Quantity>,
    pub reporting_name: String,
    pub type_of_owner: String,
    pub acquisition_or_disposition: String,
    #[serde(deserialize_with = "required_option")]
    pub direct_or_indirect: Option<String>,
    pub form_type: FormType,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub securities_transacted: Quantity,
    pub price: Price,
    pub security_name: String,
    pub url: String,
}

/// One reporting-person identity returned by reporting-name search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct InsiderReportingName {
    pub reporting_cik: Cik,
    pub reporting_name: String,
}

/// One open insider transaction-type taxonomy entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct InsiderTransactionType {
    pub transaction_type: TransactionTypeCode,
}

/// One quarterly aggregate of insider-trade activity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct InsiderTradeStatistics {
    pub symbol: Ticker,
    pub cik: Cik,
    pub year: CalendarYear,
    pub quarter: CalendarQuarter,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub acquired_transactions: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub disposed_transactions: Count,
    pub acquired_disposed_ratio: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_acquired: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_disposed: Quantity,
    pub average_acquired: f64,
    pub average_disposed: f64,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub total_purchases: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub total_sales: Count,
}

/// One beneficial-ownership acquisition filing row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BeneficialOwnershipAcquisition {
    pub cik: Cik,
    pub symbol: Ticker,
    pub filing_date: Date,
    pub accepted_date: Date,
    #[serde(deserialize_with = "required_option")]
    pub cusip: Option<Cusip>,
    pub name_of_reporting_person: String,
    #[serde(deserialize_with = "required_option")]
    pub citizenship_or_place_of_organization: Option<String>,
    pub sole_voting_power: NumericString,
    #[serde(deserialize_with = "required_option")]
    pub shared_voting_power: Option<NumericString>,
    pub sole_dispositive_power: NumericString,
    pub shared_dispositive_power: NumericString,
    pub amount_beneficially_owned: NumericString,
    pub percent_of_class: NumericString,
    pub type_of_reporting_person: String,
    pub url: String,
}
