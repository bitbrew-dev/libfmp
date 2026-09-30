//! Income-statement response models.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    types::{ApiDateTime, Cik, CurrencyCode, Date, Quantity, StatementAmount, Ticker},
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One historical or trailing-twelve-month worldwide income statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct IncomeStatement {
    pub date: Date,
    pub symbol: Ticker,
    pub reported_currency: CurrencyCode,
    pub cik: Cik,
    #[serde(deserialize_with = "required_option")]
    pub filing_date: Option<Date>,
    #[serde(deserialize_with = "required_option")]
    pub accepted_date: Option<ApiDateTime>,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub revenue: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub cost_of_revenue: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub gross_profit: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub research_and_development_expenses: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub general_and_administrative_expenses: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub selling_and_marketing_expenses: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub selling_general_and_administrative_expenses: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_expenses: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub operating_expenses: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub cost_and_expenses: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_interest_income: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub interest_income: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub interest_expense: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub depreciation_and_amortization: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub ebitda: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub ebit: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub non_operating_income_excluding_interest: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub operating_income: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_other_income_expenses_net: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub income_before_tax: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub income_tax_expense: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_income_from_continuing_operations: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_income_from_discontinued_operations: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_adjustments_to_net_income: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_income: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_income_deductions: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub bottom_line_net_income: StatementAmount,
    pub eps: f64,
    pub eps_diluted: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub weighted_average_shs_out: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub weighted_average_shs_out_dil: Quantity,
}
