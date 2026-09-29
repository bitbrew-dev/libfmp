//! Response rows returned by TipRanks add-on endpoints.
//!
//! The Python binding exposes these models under `fmp.tipranks`.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Number;

use crate::{
    codecs::IsoTimestamp,
    types::{Count, CurrencyCode, Date, Ticker, TipRanksExpertUid},
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One individual analyst rating returned by the TipRanks ratings search.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct TipRanksRatingSearchResult {
    pub symbol: Ticker,
    pub date: IsoTimestamp,
    pub recommendation_date: Date,
    #[serde(rename = "expertUID")]
    pub expert_uid: TipRanksExpertUid,
    pub analyst_name: String,
    pub firm_name: String,
    pub recommendation: String,
    pub analyst_action: String,
    pub article_title: String,
    pub article_site: String,
    pub price_target: Number,
    pub price_target_currency: CurrencyCode,
    pub url: String,
}

/// One analyst's active rating in a point-in-time symbol or analyst snapshot.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct TipRanksPointInTimeRating {
    pub symbol: Ticker,
    pub date: IsoTimestamp,
    #[serde(rename = "expertUID")]
    pub expert_uid: TipRanksExpertUid,
    pub analyst_name: String,
    pub stock_success_rate: Number,
    pub firm_name: String,
    pub last_recommendation: String,
    pub last_recommendation_date: Date,
    pub article_title: String,
    pub article_site: String,
    #[serde(deserialize_with = "required_option")]
    pub price_target: Option<Number>,
    #[serde(deserialize_with = "required_option")]
    pub price_target_currency: Option<CurrencyCode>,
    pub url: String,
    pub last_analyst_action: String,
    #[serde(deserialize_with = "required_option")]
    pub stock_return: Option<Number>,
    #[serde(deserialize_with = "required_option")]
    pub beat_target: Option<bool>,
}

/// Recommendation counts in a TipRanks ratings summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct TipRanksRecommendationCounts {
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub buy: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub hold: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub sell: Count,
}

/// Analyst-action counts in a TipRanks ratings summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct TipRanksAnalystActionCounts {
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub initiated: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub maintained: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub upgraded: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub downgraded: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub reiterated: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub resumed: Count,
}

/// Aggregated TipRanks ratings for one ticker over a date window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct TipRanksSymbolSummary {
    pub symbol: Ticker,
    pub from: Date,
    pub to: Date,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub total_recommendations: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub distinct_symbols: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub distinct_analysts: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub valid_price_targets: Count,
    pub recommendations: TipRanksRecommendationCounts,
    #[serde(rename = "analystAction")]
    pub analyst_action: TipRanksAnalystActionCounts,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub compared_price_targets: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub beats: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub misses: Count,
    pub average_return: Number,
    pub top_return: Number,
    pub worst_return: Number,
}

/// Aggregated TipRanks ratings for one analyst over a date window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct TipRanksAnalystSummary {
    #[serde(rename = "expertUID")]
    pub expert_uid: TipRanksExpertUid,
    pub from: Date,
    pub to: Date,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub total_recommendations: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub distinct_symbols: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub distinct_analysts: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub valid_price_targets: Count,
    pub recommendations: TipRanksRecommendationCounts,
    #[serde(rename = "analystAction")]
    pub analyst_action: TipRanksAnalystActionCounts,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub compared_price_targets: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub beats: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub misses: Count,
    pub average_return: Number,
    pub top_return: Number,
    pub worst_return: Number,
}

/// Aggregated TipRanks ratings for one firm over a date window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct TipRanksFirmSummary {
    pub firm_name: String,
    pub from: Date,
    pub to: Date,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub total_recommendations: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub distinct_symbols: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub distinct_analysts: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub valid_price_targets: Count,
    pub recommendations: TipRanksRecommendationCounts,
    #[serde(rename = "analystAction")]
    pub analyst_action: TipRanksAnalystActionCounts,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub compared_price_targets: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub beats: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub misses: Count,
    pub average_return: Number,
    pub top_return: Number,
    pub worst_return: Number,
}

/// One analyst profile returned by the TipRanks directory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct TipRanksAnalystProfile {
    #[serde(rename = "expertUID")]
    pub expert_uid: TipRanksExpertUid,
    pub analyst_name: String,
    pub firm_name: String,
    pub success_rate: Number,
    pub excess_return: Number,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub total_recommendations: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub good_recommendations: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub analyst_rank: Count,
    #[serde(deserialize_with = "crate::codecs::count::deserialize")]
    pub num_of_stars: Count,
}
