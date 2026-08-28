//! Income-statement-growth response model.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    types::{CurrencyCode, Date, Ticker},
};

/// One worldwide income-statement-growth row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IncomeStatementGrowth {
    pub symbol: Ticker,
    pub date: Date,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    pub reported_currency: CurrencyCode,
    pub growth_revenue: f64,
    pub growth_cost_of_revenue: f64,
    pub growth_gross_profit: f64,
    pub growth_gross_profit_ratio: f64,
    pub growth_research_and_development_expenses: f64,
    pub growth_general_and_administrative_expenses: f64,
    pub growth_selling_and_marketing_expenses: f64,
    pub growth_other_expenses: f64,
    pub growth_operating_expenses: f64,
    pub growth_cost_and_expenses: f64,
    pub growth_interest_income: f64,
    pub growth_interest_expense: f64,
    pub growth_depreciation_and_amortization: f64,
    #[serde(rename = "growthEBITDA")]
    pub growth_ebitda: f64,
    pub growth_operating_income: f64,
    pub growth_income_before_tax: f64,
    pub growth_income_tax_expense: f64,
    pub growth_net_income: f64,
    #[serde(rename = "growthEPS")]
    pub growth_eps: f64,
    #[serde(rename = "growthEPSDiluted")]
    pub growth_eps_diluted: f64,
    pub growth_weighted_average_shs_out: f64,
    pub growth_weighted_average_shs_out_dil: f64,
    #[serde(rename = "growthEBIT")]
    pub growth_ebit: f64,
    pub growth_non_operating_income_excluding_interest: f64,
    pub growth_net_interest_income: f64,
    pub growth_total_other_income_expenses_net: f64,
    pub growth_net_income_from_continuing_operations: f64,
    pub growth_other_adjustments_to_net_income: f64,
    pub growth_net_income_deductions: f64,
}
