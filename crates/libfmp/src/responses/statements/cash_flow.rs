//! Cash-flow-statement response models.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    types::{ApiDateTime, Cik, CurrencyCode, Date, StatementAmount, Ticker},
};

/// One historical or trailing-twelve-month worldwide cash-flow statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CashFlowStatement {
    pub date: Date,
    pub symbol: Ticker,
    pub reported_currency: CurrencyCode,
    pub cik: Cik,
    pub filing_date: Date,
    pub accepted_date: ApiDateTime,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_income: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub depreciation_and_amortization: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub deferred_income_tax: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub stock_based_compensation: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub change_in_working_capital: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub accounts_receivables: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub inventory: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub accounts_payables: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_working_capital: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_non_cash_items: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_cash_provided_by_operating_activities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub investments_in_property_plant_and_equipment: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub acquisitions_net: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub purchases_of_investments: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub sales_maturities_of_investments: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_investing_activities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_cash_provided_by_investing_activities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_debt_issuance: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub long_term_net_debt_issuance: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub short_term_net_debt_issuance: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_stock_issuance: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_common_stock_issuance: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub common_stock_issuance: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub common_stock_repurchased: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_preferred_stock_issuance: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_dividends_paid: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub common_dividends_paid: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub preferred_dividends_paid: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_financing_activities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_cash_provided_by_financing_activities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub effect_of_forex_changes_on_cash: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_change_in_cash: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub cash_at_end_of_period: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub cash_at_beginning_of_period: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub operating_cash_flow: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub capital_expenditure: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub free_cash_flow: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub income_taxes_paid: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub interest_paid: StatementAmount,
}
