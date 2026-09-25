//! Snapshot-like bulk response rows.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::NumericString,
    types::{CurrencyCode, Date, Isin, Ticker},
};

/// One worldwide stock-rating bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkStockRating {
    pub symbol: Ticker,
    pub date: Date,
    pub rating: String,
    pub discounted_cash_flow_score: NumericString,
    pub return_on_equity_score: NumericString,
    pub return_on_assets_score: NumericString,
    pub debt_to_equity_score: NumericString,
    pub price_to_earnings_score: NumericString,
    pub price_to_book_score: NumericString,
}

/// One worldwide discounted-cash-flow bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkDcfValuation {
    pub symbol: Ticker,
    pub date: Date,
    pub dcf: NumericString,
    #[serde(rename = "Stock Price")]
    pub stock_price: NumericString,
}

/// One worldwide financial-score bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkFinancialScore {
    pub symbol: Ticker,
    pub reported_currency: CurrencyCode,
    pub altman_z_score: NumericString,
    pub piotroski_score: NumericString,
    pub working_capital: NumericString,
    pub total_assets: NumericString,
    pub retained_earnings: NumericString,
    pub ebit: NumericString,
    pub market_cap: NumericString,
    pub total_liabilities: NumericString,
    pub revenue: NumericString,
}

/// One US price-target-summary bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkPriceTargetSummary {
    pub symbol: Ticker,
    pub last_month_count: NumericString,
    pub last_month_avg_price_target: NumericString,
    pub last_quarter_count: NumericString,
    pub last_quarter_avg_price_target: NumericString,
    pub last_year_count: NumericString,
    pub last_year_avg_price_target: NumericString,
    pub all_time_count: NumericString,
    pub all_time_avg_price_target: NumericString,
    pub publishers: String,
}

/// One worldwide ETF holding bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkEtfHolding {
    pub symbol: Ticker,
    pub name: String,
    pub shares_number: NumericString,
    pub asset: Ticker,
    pub weight_percentage: NumericString,
    pub cusip: String,
    pub isin: Isin,
    pub market_value: NumericString,
    #[serde(rename = "lastUpdated\"")]
    pub last_updated_raw: String,
}

/// One worldwide upgrades/downgrades-consensus bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkUpgradesDowngradesConsensus {
    pub symbol: String,
    pub strong_buy: NumericString,
    pub buy: NumericString,
    pub hold: NumericString,
    pub sell: NumericString,
    pub strong_sell: NumericString,
    pub consensus: String,
}
