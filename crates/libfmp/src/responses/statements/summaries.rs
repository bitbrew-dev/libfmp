//! Compact financial-summary response models.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    types::{
        ApiDateTime, CalendarYear, Count, CurrencyCode, Date, MarketCapitalization, Price,
        Quantity, StatementAmount, Ticker,
    },
};

/// One recently added financial-statement filing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct LatestFinancialStatement {
    pub symbol: Ticker,
    pub calendar_year: CalendarYear,
    pub period: FiscalPeriod,
    pub date: Date,
    pub date_added: ApiDateTime,
}

/// One company's documented financial-health scores and source values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct FinancialScore {
    pub symbol: Ticker,
    pub reported_currency: CurrencyCode,
    pub altman_z_score: f64,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub piotroski_score: Count,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub working_capital: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_assets: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub retained_earnings: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub ebit: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_cap: MarketCapitalization,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_liabilities: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub revenue: StatementAmount,
}

/// One company's owner-earnings calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct OwnerEarnings {
    pub symbol: Ticker,
    pub reported_currency: CurrencyCode,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    pub date: Date,
    #[serde(rename = "averagePPE")]
    pub average_ppe: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub maintenance_capex: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub owners_earnings: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub growth_capex: StatementAmount,
    pub owners_earnings_per_share: Price,
}

/// One historical enterprise-value calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EnterpriseValue {
    pub symbol: Ticker,
    pub date: Date,
    pub stock_price: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub number_of_shares: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_capitalization: MarketCapitalization,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub minus_cash_and_cash_equivalents: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub add_total_debt: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub enterprise_value: StatementAmount,
}
