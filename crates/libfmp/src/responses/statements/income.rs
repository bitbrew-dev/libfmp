//! Income-statement response models.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    types::{ApiDateTime, Cik, CurrencyCode, Date, Quantity, StatementAmount, Ticker},
};

/// One historical or trailing-twelve-month worldwide income statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeStatement {
    pub date: Date,
    pub symbol: Ticker,
    pub reported_currency: CurrencyCode,
    pub cik: Cik,
    pub filing_date: Date,
    pub accepted_date: ApiDateTime,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    pub revenue: StatementAmount,
    pub cost_of_revenue: StatementAmount,
    pub gross_profit: StatementAmount,
    pub research_and_development_expenses: StatementAmount,
    pub general_and_administrative_expenses: StatementAmount,
    pub selling_and_marketing_expenses: StatementAmount,
    pub selling_general_and_administrative_expenses: StatementAmount,
    pub other_expenses: StatementAmount,
    pub operating_expenses: StatementAmount,
    pub cost_and_expenses: StatementAmount,
    pub net_interest_income: StatementAmount,
    pub interest_income: StatementAmount,
    pub interest_expense: StatementAmount,
    pub depreciation_and_amortization: StatementAmount,
    pub ebitda: StatementAmount,
    pub ebit: StatementAmount,
    pub non_operating_income_excluding_interest: StatementAmount,
    pub operating_income: StatementAmount,
    pub total_other_income_expenses_net: StatementAmount,
    pub income_before_tax: StatementAmount,
    pub income_tax_expense: StatementAmount,
    pub net_income_from_continuing_operations: StatementAmount,
    pub net_income_from_discontinued_operations: StatementAmount,
    pub other_adjustments_to_net_income: StatementAmount,
    pub net_income: StatementAmount,
    pub net_income_deductions: StatementAmount,
    pub bottom_line_net_income: StatementAmount,
    pub eps: f64,
    pub eps_diluted: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub weighted_average_shs_out: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub weighted_average_shs_out_dil: Quantity,
}
