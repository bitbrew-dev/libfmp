//! Bulk income-statement and income-statement-growth rows.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::{FiscalYearString, NumericString},
    query::FiscalPeriod,
    types::{ApiDateTime, Cik, CurrencyCode, Date, Ticker},
};

/// One worldwide bulk income-statement row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkIncomeStatement {
    pub date: Date,
    pub symbol: Ticker,
    pub reported_currency: CurrencyCode,
    pub cik: Cik,
    pub filing_date: Date,
    pub accepted_date: ApiDateTime,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    pub revenue: NumericString,
    pub cost_of_revenue: NumericString,
    pub gross_profit: NumericString,
    pub research_and_development_expenses: NumericString,
    pub general_and_administrative_expenses: NumericString,
    pub selling_and_marketing_expenses: NumericString,
    pub selling_general_and_administrative_expenses: NumericString,
    pub other_expenses: NumericString,
    pub operating_expenses: NumericString,
    pub cost_and_expenses: NumericString,
    pub net_interest_income: NumericString,
    pub interest_income: NumericString,
    pub interest_expense: NumericString,
    pub depreciation_and_amortization: NumericString,
    pub ebitda: NumericString,
    pub ebit: NumericString,
    pub non_operating_income_excluding_interest: NumericString,
    pub operating_income: NumericString,
    pub total_other_income_expenses_net: NumericString,
    pub income_before_tax: NumericString,
    pub income_tax_expense: NumericString,
    pub net_income_from_continuing_operations: NumericString,
    pub net_income_from_discontinued_operations: NumericString,
    pub other_adjustments_to_net_income: NumericString,
    pub net_income: NumericString,
    pub net_income_deductions: NumericString,
    pub bottom_line_net_income: NumericString,
    pub eps: NumericString,
    pub eps_diluted: NumericString,
    pub weighted_average_shs_out: NumericString,
    pub weighted_average_shs_out_dil: NumericString,
}

/// One worldwide bulk income-statement-growth row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkIncomeStatementGrowth {
    pub symbol: Ticker,
    pub date: Date,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    pub reported_currency: CurrencyCode,
    pub growth_revenue: NumericString,
    pub growth_cost_of_revenue: NumericString,
    pub growth_gross_profit: NumericString,
    pub growth_gross_profit_ratio: NumericString,
    pub growth_research_and_development_expenses: NumericString,
    pub growth_general_and_administrative_expenses: NumericString,
    pub growth_selling_and_marketing_expenses: NumericString,
    pub growth_other_expenses: NumericString,
    pub growth_operating_expenses: NumericString,
    pub growth_cost_and_expenses: NumericString,
    pub growth_interest_income: NumericString,
    pub growth_interest_expense: NumericString,
    pub growth_depreciation_and_amortization: NumericString,
    #[serde(rename = "growthEBITDA")]
    pub growth_ebitda: NumericString,
    pub growth_operating_income: NumericString,
    pub growth_income_before_tax: NumericString,
    pub growth_income_tax_expense: NumericString,
    pub growth_net_income: NumericString,
    #[serde(rename = "growthEPS")]
    pub growth_eps: NumericString,
    #[serde(rename = "growthEPSDiluted")]
    pub growth_eps_diluted: NumericString,
    pub growth_weighted_average_shs_out: NumericString,
    pub growth_weighted_average_shs_out_dil: NumericString,
    #[serde(rename = "growthEBIT")]
    pub growth_ebit: NumericString,
    pub growth_non_operating_income_excluding_interest: NumericString,
    pub growth_net_interest_income: NumericString,
    pub growth_total_other_income_expenses_net: NumericString,
    pub growth_net_income_from_continuing_operations: NumericString,
    pub growth_other_adjustments_to_net_income: NumericString,
    pub growth_net_income_deductions: NumericString,
}
