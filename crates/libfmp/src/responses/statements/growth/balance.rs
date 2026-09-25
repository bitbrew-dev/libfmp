//! Balance-sheet-statement-growth response model.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    types::{CurrencyCode, Date, Ticker},
};

/// One worldwide balance-sheet-statement-growth row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BalanceSheetStatementGrowth {
    pub symbol: Ticker,
    pub date: Date,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    pub reported_currency: CurrencyCode,
    pub growth_cash_and_cash_equivalents: f64,
    pub growth_short_term_investments: f64,
    pub growth_cash_and_short_term_investments: f64,
    pub growth_net_receivables: f64,
    pub growth_inventory: f64,
    pub growth_other_current_assets: f64,
    pub growth_total_current_assets: f64,
    pub growth_property_plant_equipment_net: f64,
    pub growth_goodwill: f64,
    pub growth_intangible_assets: f64,
    pub growth_goodwill_and_intangible_assets: f64,
    pub growth_long_term_investments: f64,
    pub growth_tax_assets: f64,
    pub growth_other_non_current_assets: f64,
    pub growth_total_non_current_assets: f64,
    pub growth_other_assets: f64,
    pub growth_total_assets: f64,
    pub growth_account_payables: f64,
    pub growth_short_term_debt: f64,
    pub growth_tax_payables: f64,
    pub growth_deferred_revenue: f64,
    pub growth_other_current_liabilities: f64,
    pub growth_total_current_liabilities: f64,
    pub growth_long_term_debt: f64,
    pub growth_deferred_revenue_non_current: f64,
    pub growth_deferred_tax_liabilities_non_current: f64,
    pub growth_other_non_current_liabilities: f64,
    pub growth_total_non_current_liabilities: f64,
    pub growth_other_liabilities: f64,
    pub growth_total_liabilities: f64,
    pub growth_preferred_stock: f64,
    pub growth_common_stock: f64,
    pub growth_retained_earnings: f64,
    pub growth_accumulated_other_comprehensive_income_loss: f64,
    #[serde(rename = "growthOthertotalStockholdersEquity")]
    pub growth_other_total_stockholders_equity: f64,
    pub growth_total_stockholders_equity: f64,
    pub growth_minority_interest: f64,
    pub growth_total_equity: f64,
    #[serde(rename = "growthTotalLiabilitiesAndStockholdersEquity")]
    pub growth_total_liabilities_and_stockholders_equity: f64,
    pub growth_total_investments: f64,
    pub growth_total_debt: f64,
    pub growth_net_debt: f64,
    pub growth_accounts_receivables: f64,
    pub growth_other_receivables: f64,
    pub growth_prepaids: f64,
    pub growth_total_payables: f64,
    pub growth_other_payables: f64,
    pub growth_accrued_expenses: f64,
    pub growth_capital_lease_obligations_current: f64,
    pub growth_additional_paid_in_capital: f64,
    pub growth_treasury_stock: f64,
}
