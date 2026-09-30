//! Historical and trailing-twelve-month key-metrics response models.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    types::{CurrencyCode, Date, MarketCapitalization, StatementAmount, Ticker},
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One historical worldwide key-metrics row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct KeyMetrics {
    pub symbol: Ticker,
    pub date: Date,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    pub reported_currency: CurrencyCode,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_cap: MarketCapitalization,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub enterprise_value: StatementAmount,
    pub ev_to_sales: f64,
    pub ev_to_operating_cash_flow: f64,
    pub ev_to_free_cash_flow: f64,
    #[serde(rename = "evToEBITDA")]
    pub ev_to_ebitda: f64,
    #[serde(rename = "netDebtToEBITDA")]
    pub net_debt_to_ebitda: f64,
    pub current_ratio: f64,
    pub income_quality: f64,
    #[serde(deserialize_with = "required_option")]
    pub graham_number: Option<f64>,
    pub graham_net_net: f64,
    pub tax_burden: f64,
    pub interest_burden: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub working_capital: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub invested_capital: StatementAmount,
    pub return_on_assets: f64,
    pub operating_return_on_assets: f64,
    pub return_on_tangible_assets: f64,
    pub return_on_equity: f64,
    pub return_on_invested_capital: f64,
    pub return_on_capital_employed: f64,
    pub earnings_yield: f64,
    pub free_cash_flow_yield: f64,
    pub capex_to_operating_cash_flow: f64,
    pub capex_to_depreciation: f64,
    pub capex_to_revenue: f64,
    pub sales_general_and_administrative_to_revenue: f64,
    #[serde(rename = "researchAndDevelopementToRevenue")]
    pub research_and_developement_to_revenue: f64,
    pub stock_based_compensation_to_revenue: f64,
    pub intangibles_to_total_assets: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub average_receivables: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub average_payables: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub average_inventory: StatementAmount,
    pub days_of_sales_outstanding: f64,
    pub days_of_payables_outstanding: f64,
    pub days_of_inventory_outstanding: f64,
    pub operating_cycle: f64,
    pub cash_conversion_cycle: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub free_cash_flow_to_equity: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub free_cash_flow_to_firm: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub tangible_asset_value: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_current_asset_value: StatementAmount,
}

/// One worldwide trailing-twelve-month key-metrics row.
///
/// This is distinct from [`KeyMetrics`]: it has no historical date, fiscal
/// year, period, or reported currency, and its metric keys carry provider `TTM`
/// suffixes except for the documented unsuffixed `marketCap`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct KeyMetricsTtm {
    pub symbol: Ticker,
    #[serde(rename = "marketCap")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_cap: MarketCapitalization,
    #[serde(rename = "enterpriseValueTTM")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub enterprise_value_ttm: StatementAmount,
    #[serde(rename = "evToSalesTTM")]
    pub ev_to_sales_ttm: f64,
    #[serde(rename = "evToOperatingCashFlowTTM")]
    pub ev_to_operating_cash_flow_ttm: f64,
    #[serde(rename = "evToFreeCashFlowTTM")]
    pub ev_to_free_cash_flow_ttm: f64,
    #[serde(rename = "evToEBITDATTM")]
    pub ev_to_ebitda_ttm: f64,
    #[serde(rename = "netDebtToEBITDATTM")]
    pub net_debt_to_ebitda_ttm: f64,
    #[serde(rename = "currentRatioTTM")]
    pub current_ratio_ttm: f64,
    #[serde(rename = "incomeQualityTTM")]
    pub income_quality_ttm: f64,
    #[serde(deserialize_with = "required_option")]
    #[serde(rename = "grahamNumberTTM")]
    pub graham_number_ttm: Option<f64>,
    #[serde(rename = "grahamNetNetTTM")]
    pub graham_net_net_ttm: f64,
    #[serde(rename = "taxBurdenTTM")]
    pub tax_burden_ttm: f64,
    #[serde(rename = "interestBurdenTTM")]
    pub interest_burden_ttm: f64,
    #[serde(rename = "workingCapitalTTM")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub working_capital_ttm: StatementAmount,
    #[serde(rename = "investedCapitalTTM")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub invested_capital_ttm: StatementAmount,
    #[serde(rename = "returnOnAssetsTTM")]
    pub return_on_assets_ttm: f64,
    #[serde(rename = "operatingReturnOnAssetsTTM")]
    pub operating_return_on_assets_ttm: f64,
    #[serde(rename = "returnOnTangibleAssetsTTM")]
    pub return_on_tangible_assets_ttm: f64,
    #[serde(rename = "returnOnEquityTTM")]
    pub return_on_equity_ttm: f64,
    #[serde(rename = "returnOnInvestedCapitalTTM")]
    pub return_on_invested_capital_ttm: f64,
    #[serde(rename = "returnOnCapitalEmployedTTM")]
    pub return_on_capital_employed_ttm: f64,
    #[serde(rename = "earningsYieldTTM")]
    pub earnings_yield_ttm: f64,
    #[serde(rename = "freeCashFlowYieldTTM")]
    pub free_cash_flow_yield_ttm: f64,
    #[serde(rename = "capexToOperatingCashFlowTTM")]
    pub capex_to_operating_cash_flow_ttm: f64,
    #[serde(rename = "capexToDepreciationTTM")]
    pub capex_to_depreciation_ttm: f64,
    #[serde(rename = "capexToRevenueTTM")]
    pub capex_to_revenue_ttm: f64,
    #[serde(rename = "salesGeneralAndAdministrativeToRevenueTTM")]
    pub sales_general_and_administrative_to_revenue_ttm: f64,
    #[serde(rename = "researchAndDevelopementToRevenueTTM")]
    pub research_and_developement_to_revenue_ttm: f64,
    #[serde(rename = "stockBasedCompensationToRevenueTTM")]
    pub stock_based_compensation_to_revenue_ttm: f64,
    #[serde(rename = "intangiblesToTotalAssetsTTM")]
    pub intangibles_to_total_assets_ttm: f64,
    #[serde(rename = "averageReceivablesTTM")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub average_receivables_ttm: StatementAmount,
    #[serde(rename = "averagePayablesTTM")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub average_payables_ttm: StatementAmount,
    #[serde(rename = "averageInventoryTTM")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub average_inventory_ttm: StatementAmount,
    #[serde(rename = "daysOfSalesOutstandingTTM")]
    pub days_of_sales_outstanding_ttm: f64,
    #[serde(rename = "daysOfPayablesOutstandingTTM")]
    pub days_of_payables_outstanding_ttm: f64,
    #[serde(rename = "daysOfInventoryOutstandingTTM")]
    pub days_of_inventory_outstanding_ttm: f64,
    #[serde(rename = "operatingCycleTTM")]
    pub operating_cycle_ttm: f64,
    #[serde(rename = "cashConversionCycleTTM")]
    pub cash_conversion_cycle_ttm: f64,
    #[serde(rename = "freeCashFlowToEquityTTM")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub free_cash_flow_to_equity_ttm: StatementAmount,
    #[serde(rename = "freeCashFlowToFirmTTM")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub free_cash_flow_to_firm_ttm: StatementAmount,
    #[serde(rename = "tangibleAssetValueTTM")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub tangible_asset_value_ttm: StatementAmount,
    #[serde(rename = "netCurrentAssetValueTTM")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_current_asset_value_ttm: StatementAmount,
}
