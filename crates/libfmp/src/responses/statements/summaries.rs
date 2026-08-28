//! Compact financial-summary response models.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::FiscalYearString,
    query::FiscalPeriod,
    types::{
        ApiDateTime, CalendarYear, Count, CurrencyCode, Date, MarketCapitalization, Price,
        StatementAmount, Ticker,
    },
};

/// One recently added financial-statement filing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[serde(rename_all = "camelCase")]
pub struct FinancialScore {
    pub symbol: Ticker,
    pub reported_currency: CurrencyCode,
    pub altman_z_score: f64,
    pub piotroski_score: Count,
    pub working_capital: StatementAmount,
    pub total_assets: StatementAmount,
    pub retained_earnings: StatementAmount,
    pub ebit: StatementAmount,
    pub market_cap: MarketCapitalization,
    pub total_liabilities: StatementAmount,
    pub revenue: StatementAmount,
}

/// One company's owner-earnings calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerEarnings {
    pub symbol: Ticker,
    pub reported_currency: CurrencyCode,
    pub fiscal_year: FiscalYearString,
    pub period: FiscalPeriod,
    pub date: Date,
    #[serde(rename = "averagePPE")]
    pub average_ppe: f64,
    pub maintenance_capex: StatementAmount,
    pub owners_earnings: StatementAmount,
    pub growth_capex: StatementAmount,
    pub owners_earnings_per_share: Price,
}

/// One historical enterprise-value calculation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnterpriseValue {
    pub symbol: Ticker,
    pub date: Date,
    pub stock_price: Price,
    pub number_of_shares: Count,
    pub market_capitalization: MarketCapitalization,
    pub minus_cash_and_cash_equivalents: StatementAmount,
    pub add_total_debt: StatementAmount,
    pub enterprise_value: StatementAmount,
}
