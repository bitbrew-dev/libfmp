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
    pub discounted_cash_flow_score: Option<NumericString>,
    pub return_on_equity_score: Option<NumericString>,
    pub return_on_assets_score: Option<NumericString>,
    pub debt_to_equity_score: Option<NumericString>,
    pub price_to_earnings_score: Option<NumericString>,
    pub price_to_book_score: Option<NumericString>,
}

/// One worldwide discounted-cash-flow bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkDcfValuation {
    pub symbol: Ticker,
    pub date: Date,
    pub dcf: Option<NumericString>,
    #[serde(rename = "Stock Price")]
    pub stock_price: Option<NumericString>,
}

/// One worldwide financial-score bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkFinancialScore {
    pub symbol: Ticker,
    #[serde(deserialize_with = "crate::codecs::empty_or_null::deserialize")]
    pub reported_currency: Option<CurrencyCode>,
    pub altman_z_score: Option<NumericString>,
    pub piotroski_score: Option<NumericString>,
    pub working_capital: Option<NumericString>,
    pub total_assets: Option<NumericString>,
    pub retained_earnings: Option<NumericString>,
    pub ebit: Option<NumericString>,
    pub market_cap: Option<NumericString>,
    pub total_liabilities: Option<NumericString>,
    pub revenue: Option<NumericString>,
}

/// One US price-target-summary bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkPriceTargetSummary {
    pub symbol: Ticker,
    pub last_month_count: Option<NumericString>,
    pub last_month_avg_price_target: Option<NumericString>,
    pub last_quarter_count: Option<NumericString>,
    pub last_quarter_avg_price_target: Option<NumericString>,
    pub last_year_count: Option<NumericString>,
    pub last_year_avg_price_target: Option<NumericString>,
    pub all_time_count: Option<NumericString>,
    pub all_time_avg_price_target: Option<NumericString>,
    pub publishers: String,
}

/// One worldwide ETF holding bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkEtfHolding {
    pub symbol: Ticker,
    pub name: String,
    pub shares_number: Option<NumericString>,
    #[serde(deserialize_with = "crate::codecs::empty_or_null::deserialize")]
    pub asset: Option<Ticker>,
    pub weight_percentage: Option<NumericString>,
    pub cusip: String,
    #[serde(deserialize_with = "crate::codecs::empty_or_null::deserialize")]
    pub isin: Option<Isin>,
    pub market_value: Option<NumericString>,
    pub last_updated: Date,
}

/// One worldwide upgrades/downgrades-consensus bulk row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct BulkUpgradesDowngradesConsensus {
    pub symbol: String,
    pub strong_buy: Option<NumericString>,
    pub buy: Option<NumericString>,
    pub hold: Option<NumericString>,
    pub sell: Option<NumericString>,
    pub strong_sell: Option<NumericString>,
    pub consensus: String,
}
