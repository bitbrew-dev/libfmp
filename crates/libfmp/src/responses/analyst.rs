//! Response rows returned by analyst endpoints.

use serde::{Deserialize, Serialize};

use crate::types::{Count, Date, Price, StatementAmount, Ticker};

/// One dated set of analyst financial estimates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct FinancialEstimate {
    pub symbol: Ticker,
    pub date: Date,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub revenue_low: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub revenue_high: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub revenue_avg: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub ebitda_low: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub ebitda_high: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub ebitda_avg: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub ebit_low: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub ebit_high: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub ebit_avg: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_income_low: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_income_high: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub net_income_avg: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub sga_expense_low: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub sga_expense_high: StatementAmount,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub sga_expense_avg: StatementAmount,
    pub eps_avg: f64,
    pub eps_high: f64,
    pub eps_low: f64,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub num_analysts_revenue: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub num_analysts_eps: Count,
}

/// One current financial-rating snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct RatingSnapshot {
    pub symbol: Ticker,
    pub rating: String,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub overall_score: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub discounted_cash_flow_score: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub return_on_equity_score: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub return_on_assets_score: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub debt_to_equity_score: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub price_to_earnings_score: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub price_to_book_score: Count,
}

/// One dated historical financial rating.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct HistoricalRating {
    pub symbol: Ticker,
    pub date: Date,
    pub rating: String,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub overall_score: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub discounted_cash_flow_score: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub return_on_equity_score: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub return_on_assets_score: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub debt_to_equity_score: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub price_to_earnings_score: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub price_to_book_score: Count,
}

/// Price-target averages and publisher text across documented time windows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct PriceTargetSummary {
    pub symbol: Ticker,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub last_month_count: Count,
    pub last_month_avg_price_target: Price,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub last_quarter_count: Count,
    pub last_quarter_avg_price_target: Price,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub last_year_count: Count,
    pub last_year_avg_price_target: Price,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub all_time_count: Count,
    pub all_time_avg_price_target: Price,
    /// The exact provider string, whose contents happen to be JSON-array text.
    pub publishers: String,
}

/// Aggregated high, low, consensus, and median analyst price targets.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct PriceTargetConsensus {
    pub symbol: Ticker,
    pub target_high: Price,
    pub target_low: Price,
    pub target_consensus: Price,
    pub target_median: Price,
}

/// One dated stock-grade action from an analyst or institution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct StockGrade {
    pub symbol: Ticker,
    pub date: Date,
    pub grading_company: String,
    pub previous_grade: String,
    pub new_grade: String,
    pub action: String,
}

/// One dated historical count of analyst rating buckets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct HistoricalStockGrade {
    pub symbol: Ticker,
    pub date: Date,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub analyst_ratings_strong_buy: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub analyst_ratings_buy: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub analyst_ratings_hold: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub analyst_ratings_sell: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub analyst_ratings_strong_sell: Count,
}

/// Current analyst rating-bucket counts and their consensus label.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct StockGradesSummary {
    pub symbol: Ticker,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub strong_buy: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub buy: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub hold: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub sell: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub strong_sell: Count,
    pub consensus: String,
}
