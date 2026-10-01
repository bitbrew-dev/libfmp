//! Bulk cash-flow-statement and cash-flow-statement-growth rows.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::{FiscalYearString, NumericString},
    query::FiscalPeriod,
    types::{ApiDateTime, Cik, CurrencyCode, Date, Ticker},
};

/// One worldwide bulk cash-flow-statement row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkCashFlowStatement {
    pub date: Date,
    pub symbol: Ticker,
    pub reported_currency: CurrencyCode,
    pub cik: Cik,
    pub filing_date: Date,
    pub accepted_date: ApiDateTime,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    pub net_income: Option<NumericString>,
    pub depreciation_and_amortization: Option<NumericString>,
    pub deferred_income_tax: Option<NumericString>,
    pub stock_based_compensation: Option<NumericString>,
    pub change_in_working_capital: Option<NumericString>,
    pub accounts_receivables: Option<NumericString>,
    pub inventory: Option<NumericString>,
    pub accounts_payables: Option<NumericString>,
    pub other_working_capital: Option<NumericString>,
    pub other_non_cash_items: Option<NumericString>,
    pub net_cash_provided_by_operating_activities: Option<NumericString>,
    pub investments_in_property_plant_and_equipment: Option<NumericString>,
    pub acquisitions_net: Option<NumericString>,
    pub purchases_of_investments: Option<NumericString>,
    pub sales_maturities_of_investments: Option<NumericString>,
    pub other_investing_activities: Option<NumericString>,
    pub net_cash_provided_by_investing_activities: Option<NumericString>,
    pub net_debt_issuance: Option<NumericString>,
    pub long_term_net_debt_issuance: Option<NumericString>,
    pub short_term_net_debt_issuance: Option<NumericString>,
    pub net_stock_issuance: Option<NumericString>,
    pub net_common_stock_issuance: Option<NumericString>,
    pub common_stock_issuance: Option<NumericString>,
    pub common_stock_repurchased: Option<NumericString>,
    pub net_preferred_stock_issuance: Option<NumericString>,
    pub net_dividends_paid: Option<NumericString>,
    pub common_dividends_paid: Option<NumericString>,
    pub preferred_dividends_paid: Option<NumericString>,
    pub other_financing_activities: Option<NumericString>,
    pub net_cash_provided_by_financing_activities: Option<NumericString>,
    pub effect_of_forex_changes_on_cash: Option<NumericString>,
    pub net_change_in_cash: Option<NumericString>,
    pub cash_at_end_of_period: Option<NumericString>,
    pub cash_at_beginning_of_period: Option<NumericString>,
    pub operating_cash_flow: Option<NumericString>,
    pub capital_expenditure: Option<NumericString>,
    pub free_cash_flow: Option<NumericString>,
    pub income_taxes_paid: Option<NumericString>,
    pub interest_paid: Option<NumericString>,
}

/// One worldwide bulk cash-flow-statement-growth row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkCashFlowStatementGrowth {
    pub symbol: Ticker,
    pub date: Date,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    pub reported_currency: CurrencyCode,
    pub growth_net_income: Option<NumericString>,
    pub growth_depreciation_and_amortization: Option<NumericString>,
    pub growth_deferred_income_tax: Option<NumericString>,
    pub growth_stock_based_compensation: Option<NumericString>,
    pub growth_change_in_working_capital: Option<NumericString>,
    pub growth_accounts_receivables: Option<NumericString>,
    pub growth_inventory: Option<NumericString>,
    pub growth_accounts_payables: Option<NumericString>,
    pub growth_other_working_capital: Option<NumericString>,
    pub growth_other_non_cash_items: Option<NumericString>,
    #[serde(rename = "growthNetCashProvidedByOperatingActivites")]
    pub growth_net_cash_provided_by_operating_activities: Option<NumericString>,
    pub growth_investments_in_property_plant_and_equipment: Option<NumericString>,
    pub growth_acquisitions_net: Option<NumericString>,
    pub growth_purchases_of_investments: Option<NumericString>,
    pub growth_sales_maturities_of_investments: Option<NumericString>,
    #[serde(rename = "growthOtherInvestingActivites")]
    pub growth_other_investing_activities: Option<NumericString>,
    #[serde(rename = "growthNetCashUsedForInvestingActivites")]
    pub growth_net_cash_used_for_investing_activities: Option<NumericString>,
    pub growth_debt_repayment: Option<NumericString>,
    pub growth_common_stock_issued: Option<NumericString>,
    pub growth_common_stock_repurchased: Option<NumericString>,
    pub growth_dividends_paid: Option<NumericString>,
    #[serde(rename = "growthOtherFinancingActivites")]
    pub growth_other_financing_activities: Option<NumericString>,
    pub growth_net_cash_used_provided_by_financing_activities: Option<NumericString>,
    pub growth_effect_of_forex_changes_on_cash: Option<NumericString>,
    pub growth_net_change_in_cash: Option<NumericString>,
    pub growth_cash_at_end_of_period: Option<NumericString>,
    pub growth_cash_at_beginning_of_period: Option<NumericString>,
    pub growth_operating_cash_flow: Option<NumericString>,
    pub growth_capital_expenditure: Option<NumericString>,
    pub growth_free_cash_flow: Option<NumericString>,
    pub growth_net_debt_issuance: Option<NumericString>,
    pub growth_long_term_net_debt_issuance: Option<NumericString>,
    pub growth_short_term_net_debt_issuance: Option<NumericString>,
    pub growth_net_stock_issuance: Option<NumericString>,
    pub growth_preferred_dividends_paid: Option<NumericString>,
    pub growth_income_taxes_paid: Option<NumericString>,
    pub growth_interest_paid: Option<NumericString>,
}
