//! Bulk income-statement and income-statement-growth rows.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::{FiscalYearString, NumericString},
    query::FiscalPeriod,
    types::{ApiDateTime, Cik, CurrencyCode, Date, Ticker},
};

/// One worldwide bulk income-statement row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
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
    pub revenue: Option<NumericString>,
    pub cost_of_revenue: Option<NumericString>,
    pub gross_profit: Option<NumericString>,
    pub research_and_development_expenses: Option<NumericString>,
    pub general_and_administrative_expenses: Option<NumericString>,
    pub selling_and_marketing_expenses: Option<NumericString>,
    pub selling_general_and_administrative_expenses: Option<NumericString>,
    pub other_expenses: Option<NumericString>,
    pub operating_expenses: Option<NumericString>,
    pub cost_and_expenses: Option<NumericString>,
    pub net_interest_income: Option<NumericString>,
    pub interest_income: Option<NumericString>,
    pub interest_expense: Option<NumericString>,
    pub depreciation_and_amortization: Option<NumericString>,
    pub ebitda: Option<NumericString>,
    pub ebit: Option<NumericString>,
    pub non_operating_income_excluding_interest: Option<NumericString>,
    pub operating_income: Option<NumericString>,
    pub total_other_income_expenses_net: Option<NumericString>,
    pub income_before_tax: Option<NumericString>,
    pub income_tax_expense: Option<NumericString>,
    pub net_income_from_continuing_operations: Option<NumericString>,
    pub net_income_from_discontinued_operations: Option<NumericString>,
    pub other_adjustments_to_net_income: Option<NumericString>,
    pub net_income: Option<NumericString>,
    pub net_income_deductions: Option<NumericString>,
    pub bottom_line_net_income: Option<NumericString>,
    pub eps: Option<NumericString>,
    pub eps_diluted: Option<NumericString>,
    pub weighted_average_shs_out: Option<NumericString>,
    pub weighted_average_shs_out_dil: Option<NumericString>,
}

/// One worldwide bulk income-statement-growth row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkIncomeStatementGrowth {
    pub symbol: Ticker,
    pub date: Date,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    pub reported_currency: CurrencyCode,
    pub growth_revenue: Option<NumericString>,
    pub growth_cost_of_revenue: Option<NumericString>,
    pub growth_gross_profit: Option<NumericString>,
    pub growth_gross_profit_ratio: Option<NumericString>,
    pub growth_research_and_development_expenses: Option<NumericString>,
    pub growth_general_and_administrative_expenses: Option<NumericString>,
    pub growth_selling_and_marketing_expenses: Option<NumericString>,
    pub growth_other_expenses: Option<NumericString>,
    pub growth_operating_expenses: Option<NumericString>,
    pub growth_cost_and_expenses: Option<NumericString>,
    pub growth_interest_income: Option<NumericString>,
    pub growth_interest_expense: Option<NumericString>,
    pub growth_depreciation_and_amortization: Option<NumericString>,
    #[serde(rename = "growthEBITDA")]
    pub growth_ebitda: Option<NumericString>,
    pub growth_operating_income: Option<NumericString>,
    pub growth_income_before_tax: Option<NumericString>,
    pub growth_income_tax_expense: Option<NumericString>,
    pub growth_net_income: Option<NumericString>,
    #[serde(rename = "growthEPS")]
    pub growth_eps: Option<NumericString>,
    #[serde(rename = "growthEPSDiluted")]
    pub growth_eps_diluted: Option<NumericString>,
    pub growth_weighted_average_shs_out: Option<NumericString>,
    pub growth_weighted_average_shs_out_dil: Option<NumericString>,
    #[serde(rename = "growthEBIT")]
    pub growth_ebit: Option<NumericString>,
    pub growth_non_operating_income_excluding_interest: Option<NumericString>,
    pub growth_net_interest_income: Option<NumericString>,
    pub growth_total_other_income_expenses_net: Option<NumericString>,
    pub growth_net_income_from_continuing_operations: Option<NumericString>,
    pub growth_other_adjustments_to_net_income: Option<NumericString>,
    pub growth_net_income_deductions: Option<NumericString>,
}
