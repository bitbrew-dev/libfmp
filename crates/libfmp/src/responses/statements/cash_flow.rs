//! Cash-flow-statement response models.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    types::{ApiDateTime, Cik, CurrencyCode, Date, StatementAmount, Ticker},
};

/// One historical or trailing-twelve-month worldwide cash-flow statement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub net_income: StatementAmount,
    pub depreciation_and_amortization: StatementAmount,
    pub deferred_income_tax: StatementAmount,
    pub stock_based_compensation: StatementAmount,
    pub change_in_working_capital: StatementAmount,
    pub accounts_receivables: StatementAmount,
    pub inventory: StatementAmount,
    pub accounts_payables: StatementAmount,
    pub other_working_capital: StatementAmount,
    pub other_non_cash_items: StatementAmount,
    pub net_cash_provided_by_operating_activities: StatementAmount,
    pub investments_in_property_plant_and_equipment: StatementAmount,
    pub acquisitions_net: StatementAmount,
    pub purchases_of_investments: StatementAmount,
    pub sales_maturities_of_investments: StatementAmount,
    pub other_investing_activities: StatementAmount,
    pub net_cash_provided_by_investing_activities: StatementAmount,
    pub net_debt_issuance: StatementAmount,
    pub long_term_net_debt_issuance: StatementAmount,
    pub short_term_net_debt_issuance: StatementAmount,
    pub net_stock_issuance: StatementAmount,
    pub net_common_stock_issuance: StatementAmount,
    pub common_stock_issuance: StatementAmount,
    pub common_stock_repurchased: StatementAmount,
    pub net_preferred_stock_issuance: StatementAmount,
    pub net_dividends_paid: StatementAmount,
    pub common_dividends_paid: StatementAmount,
    pub preferred_dividends_paid: StatementAmount,
    pub other_financing_activities: StatementAmount,
    pub net_cash_provided_by_financing_activities: StatementAmount,
    pub effect_of_forex_changes_on_cash: StatementAmount,
    pub net_change_in_cash: StatementAmount,
    pub cash_at_end_of_period: StatementAmount,
    pub cash_at_beginning_of_period: StatementAmount,
    pub operating_cash_flow: StatementAmount,
    pub capital_expenditure: StatementAmount,
    pub free_cash_flow: StatementAmount,
    pub income_taxes_paid: StatementAmount,
    pub interest_paid: StatementAmount,
}
