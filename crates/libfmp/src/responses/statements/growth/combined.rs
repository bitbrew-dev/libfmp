//! Combined financial-statement growth response model.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    types::{CurrencyCode, Date, Ticker},
};

/// One combined financial-statement growth row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct FinancialStatementGrowth {
    pub symbol: Ticker,
    pub date: Date,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    pub reported_currency: CurrencyCode,
    pub revenue_growth: f64,
    pub gross_profit_growth: f64,
    #[serde(rename = "ebitgrowth")]
    pub ebit_growth: f64,
    pub operating_income_growth: f64,
    pub net_income_growth: f64,
    #[serde(rename = "epsgrowth")]
    pub eps_growth: f64,
    #[serde(rename = "epsdilutedGrowth")]
    pub eps_diluted_growth: f64,
    pub weighted_average_shares_growth: f64,
    pub weighted_average_shares_diluted_growth: f64,
    pub dividends_per_share_growth: f64,
    pub operating_cash_flow_growth: f64,
    pub receivables_growth: f64,
    pub inventory_growth: f64,
    pub asset_growth: f64,
    #[serde(rename = "bookValueperShareGrowth")]
    pub book_value_per_share_growth: f64,
    pub debt_growth: f64,
    #[serde(rename = "rdexpenseGrowth")]
    pub rd_expense_growth: f64,
    #[serde(rename = "sgaexpensesGrowth")]
    pub sga_expenses_growth: f64,
    pub free_cash_flow_growth: f64,
    pub ten_y_revenue_growth_per_share: f64,
    pub five_y_revenue_growth_per_share: f64,
    pub three_y_revenue_growth_per_share: f64,
    #[serde(rename = "tenYOperatingCFGrowthPerShare")]
    pub ten_y_operating_cf_growth_per_share: f64,
    #[serde(rename = "fiveYOperatingCFGrowthPerShare")]
    pub five_y_operating_cf_growth_per_share: f64,
    #[serde(rename = "threeYOperatingCFGrowthPerShare")]
    pub three_y_operating_cf_growth_per_share: f64,
    pub ten_y_net_income_growth_per_share: f64,
    pub five_y_net_income_growth_per_share: f64,
    pub three_y_net_income_growth_per_share: f64,
    pub ten_y_shareholders_equity_growth_per_share: f64,
    pub five_y_shareholders_equity_growth_per_share: f64,
    pub three_y_shareholders_equity_growth_per_share: f64,
    #[serde(rename = "tenYDividendperShareGrowthPerShare")]
    pub ten_y_dividend_per_share_growth_per_share: f64,
    #[serde(rename = "fiveYDividendperShareGrowthPerShare")]
    pub five_y_dividend_per_share_growth_per_share: f64,
    #[serde(rename = "threeYDividendperShareGrowthPerShare")]
    pub three_y_dividend_per_share_growth_per_share: f64,
    pub ebitda_growth: f64,
    pub growth_capital_expenditure: f64,
    pub ten_y_bottom_line_net_income_growth_per_share: f64,
    pub five_y_bottom_line_net_income_growth_per_share: f64,
    pub three_y_bottom_line_net_income_growth_per_share: f64,
}
