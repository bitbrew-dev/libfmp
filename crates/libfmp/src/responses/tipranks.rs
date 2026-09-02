//! Response rows returned by TipRanks add-on endpoints.
//!
//! Future Python bindings reserve these models under `fmp.tipranks`.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Number;

use crate::{
    codecs::IsoTimestamp,
    types::{CurrencyCode, Date, Ticker, TipRanksExpertUid},
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
