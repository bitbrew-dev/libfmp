//! Bulk financial-metric, peer, and earnings-surprise rows.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::NumericString,
    types::{Date, Ticker},
};

/// One worldwide key-metrics trailing-twelve-month bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkKeyMetricsTtm {
    pub symbol: Ticker,
    #[serde(rename = "marketCap")]
    pub market_cap: NumericString,
    #[serde(rename = "enterpriseValueTTM")]
    pub enterprise_value_ttm: NumericString,
    #[serde(rename = "evToSalesTTM")]
    pub ev_to_sales_ttm: NumericString,
    #[serde(rename = "evToOperatingCashFlowTTM")]
    pub ev_to_operating_cash_flow_ttm: NumericString,
    #[serde(rename = "evToFreeCashFlowTTM")]
    pub ev_to_free_cash_flow_ttm: NumericString,
    #[serde(rename = "evToEBITDATTM")]
    pub ev_to_ebitda_ttm: NumericString,
    #[serde(rename = "netDebtToEBITDATTM")]
    pub net_debt_to_ebitda_ttm: NumericString,
    #[serde(rename = "currentRatioTTM")]
    pub current_ratio_ttm: NumericString,
    #[serde(rename = "incomeQualityTTM")]
    pub income_quality_ttm: NumericString,
    #[serde(rename = "grahamNumberTTM")]
    pub graham_number_ttm: NumericString,
    #[serde(rename = "grahamNetNetTTM")]
    pub graham_net_net_ttm: NumericString,
    #[serde(rename = "taxBurdenTTM")]
    pub tax_burden_ttm: NumericString,
    #[serde(rename = "interestBurdenTTM")]
    pub interest_burden_ttm: NumericString,
    #[serde(rename = "workingCapitalTTM")]
    pub working_capital_ttm: NumericString,
    #[serde(rename = "investedCapitalTTM")]
    pub invested_capital_ttm: NumericString,
    #[serde(rename = "returnOnAssetsTTM")]
    pub return_on_assets_ttm: NumericString,
    #[serde(rename = "operatingReturnOnAssetsTTM")]
    pub operating_return_on_assets_ttm: NumericString,
    #[serde(rename = "returnOnTangibleAssetsTTM")]
    pub return_on_tangible_assets_ttm: NumericString,
    #[serde(rename = "returnOnEquityTTM")]
    pub return_on_equity_ttm: NumericString,
    #[serde(rename = "returnOnInvestedCapitalTTM")]
    pub return_on_invested_capital_ttm: NumericString,
    #[serde(rename = "returnOnCapitalEmployedTTM")]
    pub return_on_capital_employed_ttm: NumericString,
    #[serde(rename = "earningsYieldTTM")]
    pub earnings_yield_ttm: NumericString,
    #[serde(rename = "freeCashFlowYieldTTM")]
    pub free_cash_flow_yield_ttm: NumericString,
    #[serde(rename = "capexToOperatingCashFlowTTM")]
    pub capex_to_operating_cash_flow_ttm: NumericString,
    #[serde(rename = "capexToDepreciationTTM")]
    pub capex_to_depreciation_ttm: NumericString,
    #[serde(rename = "capexToRevenueTTM")]
    pub capex_to_revenue_ttm: NumericString,
    #[serde(rename = "salesGeneralAndAdministrativeToRevenueTTM")]
    pub sales_general_and_administrative_to_revenue_ttm: NumericString,
    #[serde(rename = "researchAndDevelopementToRevenueTTM")]
    pub research_and_development_to_revenue_ttm: NumericString,
    #[serde(rename = "stockBasedCompensationToRevenueTTM")]
    pub stock_based_compensation_to_revenue_ttm: NumericString,
    #[serde(rename = "intangiblesToTotalAssetsTTM")]
    pub intangibles_to_total_assets_ttm: NumericString,
    #[serde(rename = "averageReceivablesTTM")]
    pub average_receivables_ttm: NumericString,
    #[serde(rename = "averagePayablesTTM")]
    pub average_payables_ttm: NumericString,
    #[serde(rename = "averageInventoryTTM")]
    pub average_inventory_ttm: NumericString,
    #[serde(rename = "daysOfSalesOutstandingTTM")]
    pub days_of_sales_outstanding_ttm: NumericString,
    #[serde(rename = "daysOfPayablesOutstandingTTM")]
    pub days_of_payables_outstanding_ttm: NumericString,
    #[serde(rename = "daysOfInventoryOutstandingTTM")]
    pub days_of_inventory_outstanding_ttm: NumericString,
    #[serde(rename = "operatingCycleTTM")]
    pub operating_cycle_ttm: NumericString,
    #[serde(rename = "cashConversionCycleTTM")]
    pub cash_conversion_cycle_ttm: NumericString,
    #[serde(rename = "freeCashFlowToEquityTTM")]
    pub free_cash_flow_to_equity_ttm: NumericString,
    #[serde(rename = "freeCashFlowToFirmTTM")]
    pub free_cash_flow_to_firm_ttm: NumericString,
    #[serde(rename = "tangibleAssetValueTTM")]
    pub tangible_asset_value_ttm: NumericString,
    #[serde(rename = "netCurrentAssetValueTTM")]
    pub net_current_asset_value_ttm: NumericString,
}

/// One worldwide financial-ratios trailing-twelve-month bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkFinancialRatiosTtm {
    pub symbol: Ticker,
    #[serde(rename = "grossProfitMarginTTM")]
    pub gross_profit_margin_ttm: NumericString,
    #[serde(rename = "ebitMarginTTM")]
    pub ebit_margin_ttm: NumericString,
    #[serde(rename = "ebitdaMarginTTM")]
    pub ebitda_margin_ttm: NumericString,
    #[serde(rename = "operatingProfitMarginTTM")]
    pub operating_profit_margin_ttm: NumericString,
    #[serde(rename = "pretaxProfitMarginTTM")]
    pub pretax_profit_margin_ttm: NumericString,
    #[serde(rename = "continuousOperationsProfitMarginTTM")]
    pub continuous_operations_profit_margin_ttm: NumericString,
    #[serde(rename = "netProfitMarginTTM")]
    pub net_profit_margin_ttm: NumericString,
    #[serde(rename = "bottomLineProfitMarginTTM")]
    pub bottom_line_profit_margin_ttm: NumericString,
    #[serde(rename = "receivablesTurnoverTTM")]
    pub receivables_turnover_ttm: NumericString,
    #[serde(rename = "payablesTurnoverTTM")]
    pub payables_turnover_ttm: NumericString,
    #[serde(rename = "inventoryTurnoverTTM")]
    pub inventory_turnover_ttm: NumericString,
    #[serde(rename = "fixedAssetTurnoverTTM")]
    pub fixed_asset_turnover_ttm: NumericString,
    #[serde(rename = "assetTurnoverTTM")]
    pub asset_turnover_ttm: NumericString,
    #[serde(rename = "currentRatioTTM")]
    pub current_ratio_ttm: NumericString,
    #[serde(rename = "quickRatioTTM")]
    pub quick_ratio_ttm: NumericString,
    #[serde(rename = "solvencyRatioTTM")]
    pub solvency_ratio_ttm: NumericString,
    #[serde(rename = "cashRatioTTM")]
    pub cash_ratio_ttm: NumericString,
    #[serde(rename = "priceToEarningsRatioTTM")]
    pub price_to_earnings_ratio_ttm: NumericString,
    #[serde(rename = "priceToEarningsGrowthRatioTTM")]
    pub price_to_earnings_growth_ratio_ttm: NumericString,
    #[serde(rename = "forwardPriceToEarningsGrowthRatioTTM")]
    pub forward_price_to_earnings_growth_ratio_ttm: NumericString,
    #[serde(rename = "priceToBookRatioTTM")]
    pub price_to_book_ratio_ttm: NumericString,
    #[serde(rename = "priceToSalesRatioTTM")]
    pub price_to_sales_ratio_ttm: NumericString,
    #[serde(rename = "priceToFreeCashFlowRatioTTM")]
    pub price_to_free_cash_flow_ratio_ttm: NumericString,
    #[serde(rename = "priceToOperatingCashFlowRatioTTM")]
    pub price_to_operating_cash_flow_ratio_ttm: NumericString,
    #[serde(rename = "debtToAssetsRatioTTM")]
    pub debt_to_assets_ratio_ttm: NumericString,
    #[serde(rename = "debtToEquityRatioTTM")]
    pub debt_to_equity_ratio_ttm: NumericString,
    #[serde(rename = "debtToCapitalRatioTTM")]
    pub debt_to_capital_ratio_ttm: NumericString,
    #[serde(rename = "longTermDebtToCapitalRatioTTM")]
    pub long_term_debt_to_capital_ratio_ttm: NumericString,
    #[serde(rename = "financialLeverageRatioTTM")]
    pub financial_leverage_ratio_ttm: NumericString,
    #[serde(rename = "workingCapitalTurnoverRatioTTM")]
    pub working_capital_turnover_ratio_ttm: NumericString,
    #[serde(rename = "operatingCashFlowRatioTTM")]
    pub operating_cash_flow_ratio_ttm: NumericString,
    #[serde(rename = "operatingCashFlowSalesRatioTTM")]
    pub operating_cash_flow_sales_ratio_ttm: NumericString,
    #[serde(rename = "freeCashFlowOperatingCashFlowRatioTTM")]
    pub free_cash_flow_operating_cash_flow_ratio_ttm: NumericString,
    #[serde(rename = "debtServiceCoverageRatioTTM")]
    pub debt_service_coverage_ratio_ttm: NumericString,
    #[serde(rename = "interestCoverageRatioTTM")]
    pub interest_coverage_ratio_ttm: NumericString,
    #[serde(rename = "shortTermOperatingCashFlowCoverageRatioTTM")]
    pub short_term_operating_cash_flow_coverage_ratio_ttm: NumericString,
    #[serde(rename = "operatingCashFlowCoverageRatioTTM")]
    pub operating_cash_flow_coverage_ratio_ttm: NumericString,
    #[serde(rename = "capitalExpenditureCoverageRatioTTM")]
    pub capital_expenditure_coverage_ratio_ttm: NumericString,
    #[serde(rename = "dividendPaidAndCapexCoverageRatioTTM")]
    pub dividend_paid_and_capex_coverage_ratio_ttm: NumericString,
    #[serde(rename = "dividendPayoutRatioTTM")]
    pub dividend_payout_ratio_ttm: NumericString,
    #[serde(rename = "dividendYieldTTM")]
    pub dividend_yield_ttm: NumericString,
    #[serde(rename = "enterpriseValueTTM")]
    pub enterprise_value_ttm: NumericString,
    #[serde(rename = "revenuePerShareTTM")]
    pub revenue_per_share_ttm: NumericString,
    #[serde(rename = "netIncomePerShareTTM")]
    pub net_income_per_share_ttm: NumericString,
    #[serde(rename = "interestDebtPerShareTTM")]
    pub interest_debt_per_share_ttm: NumericString,
    #[serde(rename = "cashPerShareTTM")]
    pub cash_per_share_ttm: NumericString,
    #[serde(rename = "bookValuePerShareTTM")]
    pub book_value_per_share_ttm: NumericString,
    #[serde(rename = "tangibleBookValuePerShareTTM")]
    pub tangible_book_value_per_share_ttm: NumericString,
    #[serde(rename = "shareholdersEquityPerShareTTM")]
    pub shareholders_equity_per_share_ttm: NumericString,
    #[serde(rename = "operatingCashFlowPerShareTTM")]
    pub operating_cash_flow_per_share_ttm: NumericString,
    #[serde(rename = "capexPerShareTTM")]
    pub capex_per_share_ttm: NumericString,
    #[serde(rename = "freeCashFlowPerShareTTM")]
    pub free_cash_flow_per_share_ttm: NumericString,
    #[serde(rename = "netIncomePerEBTTTM")]
    pub net_income_per_ebt_ttm: NumericString,
    #[serde(rename = "ebtPerEbitTTM")]
    pub ebt_per_ebit_ttm: NumericString,
    #[serde(rename = "priceToFairValueTTM")]
    pub price_to_fair_value_ttm: NumericString,
    #[serde(rename = "debtToMarketCapTTM")]
    pub debt_to_market_cap_ttm: NumericString,
    #[serde(rename = "effectiveTaxRateTTM")]
    pub effective_tax_rate_ttm: NumericString,
    #[serde(rename = "enterpriseValueMultipleTTM")]
    pub enterprise_value_multiple_ttm: NumericString,
    #[serde(rename = "dividendPerShareTTM")]
    pub dividend_per_share_ttm: NumericString,
}

/// One worldwide stock-peer bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BulkStockPeers {
    pub symbol: Ticker,
    pub peers: String,
}

/// One worldwide annual earnings-surprise bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkEarningsSurprise {
    pub symbol: Ticker,
    pub date: Date,
    pub eps_actual: NumericString,
    pub eps_estimated: NumericString,
    pub last_updated: Date,
}
