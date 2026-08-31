//! Response rows returned by insider-trading endpoints.
//!
//! Future Python bindings reserve these models under `fmp.insider_trading`.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::NumericString,
    types::{
        CalendarQuarter, CalendarYear, Cik, Count, Cusip, Date, FormType, Price, Ticker,
        TransactionTypeCode,
    },
};

/// One insider trade shared by the latest and search feeds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsiderTrade {
    pub symbol: Ticker,
    pub filing_date: Date,
    pub transaction_date: Date,
    pub reporting_cik: Cik,
    pub company_cik: Cik,
    pub transaction_type: TransactionTypeCode,
    pub securities_owned: Count,
    pub reporting_name: String,
    pub type_of_owner: String,
    pub acquisition_or_disposition: String,
    pub direct_or_indirect: String,
    pub form_type: FormType,
    pub securities_transacted: Count,
    pub price: Price,
    pub security_name: String,
    pub url: String,
}

/// One reporting-person identity returned by reporting-name search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsiderReportingName {
    pub reporting_cik: Cik,
    pub reporting_name: String,
}

/// One open insider transaction-type taxonomy entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsiderTransactionType {
    pub transaction_type: TransactionTypeCode,
}

/// One quarterly aggregate of insider-trade activity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InsiderTradeStatistics {
    pub symbol: Ticker,
    pub cik: Cik,
    pub year: CalendarYear,
    pub quarter: CalendarQuarter,
    pub acquired_transactions: Count,
    pub disposed_transactions: Count,
    pub acquired_disposed_ratio: f64,
    pub total_acquired: Count,
    pub total_disposed: Count,
    pub average_acquired: f64,
    pub average_disposed: f64,
    pub total_purchases: Count,
    pub total_sales: Count,
}

/// One beneficial-ownership acquisition filing row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeneficialOwnershipAcquisition {
    pub cik: Cik,
    pub symbol: Ticker,
    pub filing_date: Date,
    pub accepted_date: Date,
    pub cusip: Cusip,
    pub name_of_reporting_person: String,
    pub citizenship_or_place_of_organization: String,
    pub sole_voting_power: NumericString,
    pub shared_voting_power: NumericString,
    pub sole_dispositive_power: NumericString,
    pub shared_dispositive_power: NumericString,
    pub amount_beneficially_owned: NumericString,
    pub percent_of_class: NumericString,
    pub type_of_reporting_person: String,
    pub url: String,
}
