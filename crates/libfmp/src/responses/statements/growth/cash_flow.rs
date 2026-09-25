//! Cash-flow-statement growth response model.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    types::{CurrencyCode, Date, Ticker},
};

/// One historical cash-flow-statement growth row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CashFlowStatementGrowth {
    pub symbol: Ticker,
    pub date: Date,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    pub reported_currency: CurrencyCode,
    pub growth_net_income: f64,
    pub growth_depreciation_and_amortization: f64,
    pub growth_deferred_income_tax: f64,
    pub growth_stock_based_compensation: f64,
    pub growth_change_in_working_capital: f64,
    pub growth_accounts_receivables: f64,
    pub growth_inventory: f64,
    pub growth_accounts_payables: f64,
    pub growth_other_working_capital: f64,
    pub growth_other_non_cash_items: f64,
    #[serde(rename = "growthNetCashProvidedByOperatingActivites")]
    pub growth_net_cash_provided_by_operating_activities: f64,
    pub growth_investments_in_property_plant_and_equipment: f64,
    pub growth_acquisitions_net: f64,
    pub growth_purchases_of_investments: f64,
    pub growth_sales_maturities_of_investments: f64,
    #[serde(rename = "growthOtherInvestingActivites")]
    pub growth_other_investing_activities: f64,
    #[serde(rename = "growthNetCashUsedForInvestingActivites")]
    pub growth_net_cash_used_for_investing_activities: f64,
    pub growth_debt_repayment: f64,
    pub growth_common_stock_issued: f64,
    pub growth_common_stock_repurchased: f64,
    pub growth_dividends_paid: f64,
    #[serde(rename = "growthOtherFinancingActivites")]
    pub growth_other_financing_activities: f64,
    pub growth_net_cash_used_provided_by_financing_activities: f64,
    pub growth_effect_of_forex_changes_on_cash: f64,
    pub growth_net_change_in_cash: f64,
    pub growth_cash_at_end_of_period: f64,
    pub growth_cash_at_beginning_of_period: f64,
    pub growth_operating_cash_flow: f64,
    pub growth_capital_expenditure: f64,
    pub growth_free_cash_flow: f64,
    pub growth_net_debt_issuance: f64,
    pub growth_long_term_net_debt_issuance: f64,
    pub growth_short_term_net_debt_issuance: f64,
    pub growth_net_stock_issuance: f64,
    pub growth_preferred_dividends_paid: f64,
    pub growth_income_taxes_paid: f64,
    pub growth_interest_paid: f64,
}
