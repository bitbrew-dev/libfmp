//! Cash-flow-statement response models.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    types::{ApiDateTime, Cik, CurrencyCode, Date, StatementAmount, Ticker},
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

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
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub deferred_income_tax: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub stock_based_compensation: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub change_in_working_capital: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub accounts_receivables: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub inventory: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub accounts_payables: Option<StatementAmount>,
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
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub purchases_of_investments: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub sales_maturities_of_investments: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_investing_activities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_cash_provided_by_investing_activities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_debt_issuance: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub long_term_net_debt_issuance: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub short_term_net_debt_issuance: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub net_stock_issuance: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub net_common_stock_issuance: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub common_stock_issuance: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub common_stock_repurchased: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub net_preferred_stock_issuance: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub net_dividends_paid: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub common_dividends_paid: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub preferred_dividends_paid: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_financing_activities: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub net_cash_provided_by_financing_activities: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub effect_of_forex_changes_on_cash: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_change_in_cash: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub cash_at_end_of_period: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub cash_at_beginning_of_period: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub operating_cash_flow: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub capital_expenditure: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub free_cash_flow: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub income_taxes_paid: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub interest_paid: Option<StatementAmount>,
}
