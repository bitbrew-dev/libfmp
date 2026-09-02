//! Response rows returned by TipRanks add-on endpoints.
//!
//! Future Python bindings reserve these models under `fmp.tipranks`.

use serde::{Deserialize, Serialize};
use serde_json::Number;

use crate::{
    codecs::IsoTimestamp,
    types::{CurrencyCode, Date, Ticker, TipRanksExpertUid},
};

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
