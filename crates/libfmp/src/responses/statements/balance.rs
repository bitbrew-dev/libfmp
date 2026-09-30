//! Balance-sheet response models.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    types::{ApiDateTime, Cik, CurrencyCode, Date, StatementAmount, Ticker},
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One historical worldwide balance-sheet statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BalanceSheetStatement {
    pub date: Date,
    pub symbol: Ticker,
    pub reported_currency: CurrencyCode,
    pub cik: Cik,
    pub filing_date: Date,
    pub accepted_date: ApiDateTime,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub cash_and_cash_equivalents: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub short_term_investments: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub cash_and_short_term_investments: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub net_receivables: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub accounts_receivables: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_receivables: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub inventory: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub prepaids: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_current_assets: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub total_current_assets: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub property_plant_equipment_net: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub goodwill: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub intangible_assets: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub goodwill_and_intangible_assets: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub long_term_investments: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub tax_assets: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_non_current_assets: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub total_non_current_assets: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_assets: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub total_assets: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_payables: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub account_payables: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub other_payables: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub accrued_expenses: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub short_term_debt: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub capital_lease_obligations_current: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub tax_payables: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub deferred_revenue: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_current_liabilities: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub total_current_liabilities: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub long_term_debt: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub capital_lease_obligations_non_current: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub deferred_revenue_non_current: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub deferred_tax_liabilities_non_current: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_non_current_liabilities: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub total_non_current_liabilities: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_liabilities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub capital_lease_obligations: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub total_liabilities: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub treasury_stock: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub preferred_stock: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub common_stock: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub retained_earnings: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub additional_paid_in_capital: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub accumulated_other_comprehensive_income_loss: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_total_stockholders_equity: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub total_stockholders_equity: Option<StatementAmount>,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub total_equity: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub minority_interest: StatementAmount,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub total_liabilities_and_total_equity: Option<StatementAmount>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_investments: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_debt: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_debt: StatementAmount,
}

/// One trailing-twelve-month worldwide balance-sheet statement.
///
/// The provider's documented TTM row omits
/// `capitalLeaseObligationsNonCurrent`, so this is intentionally distinct from
/// [`BalanceSheetStatement`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BalanceSheetStatementTtm {
    pub date: Date,
    pub symbol: Ticker,
    pub reported_currency: CurrencyCode,
    pub cik: Cik,
    pub filing_date: Date,
    pub accepted_date: ApiDateTime,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub cash_and_cash_equivalents: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub short_term_investments: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub cash_and_short_term_investments: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_receivables: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub accounts_receivables: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_receivables: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub inventory: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub prepaids: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_current_assets: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_current_assets: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub property_plant_equipment_net: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub goodwill: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub intangible_assets: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub goodwill_and_intangible_assets: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub long_term_investments: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub tax_assets: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_non_current_assets: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_non_current_assets: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_assets: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_assets: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_payables: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub account_payables: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_payables: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub accrued_expenses: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub short_term_debt: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub capital_lease_obligations_current: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub tax_payables: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub deferred_revenue: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_current_liabilities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_current_liabilities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub long_term_debt: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub deferred_revenue_non_current: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub deferred_tax_liabilities_non_current: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_non_current_liabilities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_non_current_liabilities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_liabilities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub capital_lease_obligations: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_liabilities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub treasury_stock: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub preferred_stock: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub common_stock: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub retained_earnings: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub additional_paid_in_capital: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub accumulated_other_comprehensive_income_loss: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub other_total_stockholders_equity: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_stockholders_equity: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_equity: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub minority_interest: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_liabilities_and_total_equity: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_investments: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_debt: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_debt: StatementAmount,
}
