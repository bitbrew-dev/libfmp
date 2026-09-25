//! Response models owned by discounted-cash-flow valuation endpoints.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::FiscalYearString,
    types::{Date, Percentage, Price, Quantity, StatementAmount, Ticker},
};

/// One standard or levered discounted-cash-flow valuation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DcfValuation {
    pub symbol: Ticker,
    pub date: Date,
    pub dcf: f64,
    #[serde(rename = "Stock Price")]
    pub stock_price: f64,
}

/// One detailed valuation returned by the custom unlevered DCF route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomDcfValuation {
    pub year: FiscalYearString,
    pub symbol: Ticker,
    pub revenue: StatementAmount,
    pub revenue_percentage: Percentage,
    pub ebitda: StatementAmount,
    pub ebitda_percentage: Percentage,
    pub ebit: StatementAmount,
    pub ebit_percentage: Percentage,
    pub depreciation: StatementAmount,
    pub depreciation_percentage: Percentage,
    pub total_cash: StatementAmount,
    pub total_cash_percentage: Percentage,
    pub receivables: StatementAmount,
    pub receivables_percentage: Percentage,
    pub inventories: StatementAmount,
    pub inventories_percentage: Percentage,
    pub payable: StatementAmount,
    pub payable_percentage: Percentage,
    pub capital_expenditure: StatementAmount,
    pub capital_expenditure_percentage: Percentage,
    pub price: Price,
    pub beta: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub diluted_shares_outstanding: Quantity,
    #[serde(rename = "costofDebt")]
    pub cost_of_debt: Percentage,
    pub tax_rate: Percentage,
    pub after_tax_cost_of_debt: Percentage,
    pub risk_free_rate: Percentage,
    pub market_risk_premium: Percentage,
    pub cost_of_equity: Percentage,
    pub total_debt: StatementAmount,
    pub total_equity: StatementAmount,
    pub total_capital: StatementAmount,
    pub debt_weighting: Percentage,
    pub equity_weighting: Percentage,
    pub wacc: Percentage,
    pub tax_rate_cash: StatementAmount,
    pub ebiat: StatementAmount,
    pub ufcf: StatementAmount,
    pub sum_pv_ufcf: StatementAmount,
    pub long_term_growth_rate: Percentage,
    pub terminal_value: StatementAmount,
    pub present_terminal_value: StatementAmount,
    pub enterprise_value: StatementAmount,
    pub net_debt: StatementAmount,
    pub equity_value: StatementAmount,
    pub equity_value_per_share: Price,
    pub free_cash_flow_t1: StatementAmount,
}

/// One detailed valuation returned by the custom levered DCF route.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomLeveredDcfValuation {
    pub year: FiscalYearString,
    pub symbol: Ticker,
    pub revenue: StatementAmount,
    pub revenue_percentage: Percentage,
    pub capital_expenditure: StatementAmount,
    pub capital_expenditure_percentage: Percentage,
    pub price: Price,
    pub beta: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub diluted_shares_outstanding: Quantity,
    #[serde(rename = "costofDebt")]
    pub cost_of_debt: Percentage,
    pub tax_rate: Percentage,
    pub after_tax_cost_of_debt: Percentage,
    pub risk_free_rate: Percentage,
    pub market_risk_premium: Percentage,
    pub cost_of_equity: Percentage,
    pub total_debt: StatementAmount,
    pub total_equity: StatementAmount,
    pub total_capital: StatementAmount,
    pub debt_weighting: Percentage,
    pub equity_weighting: Percentage,
    pub wacc: Percentage,
    pub operating_cash_flow: StatementAmount,
    pub pv_lfcf: StatementAmount,
    pub sum_pv_lfcf: StatementAmount,
    pub long_term_growth_rate: Percentage,
    pub free_cash_flow: StatementAmount,
    pub terminal_value: StatementAmount,
    pub present_terminal_value: StatementAmount,
    pub enterprise_value: StatementAmount,
    pub net_debt: StatementAmount,
    pub equity_value: StatementAmount,
    pub equity_value_per_share: Price,
    pub free_cash_flow_t1: StatementAmount,
    pub operating_cash_flow_percentage: Percentage,
}
