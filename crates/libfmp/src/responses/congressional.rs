//! Response rows returned by congressional financial-disclosure endpoints.
//!
//! Future Python bindings reserve these models under `fmp.congressional`.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::TitleCaseBoolFlag,
    types::{CongressionalMemberId, Date},
};

/// One financial trade disclosed by a member of Congress.
///
/// The provider calls the cross-chamber member identifier `senateID`, even for
/// House members. `symbol` deliberately remains a string because the
/// documented Senate-by-name response contains an empty value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CongressionalTrade {
    pub symbol: String,
    #[serde(rename = "senateID")]
    pub member_id: CongressionalMemberId,
    pub disclosure_date: Date,
    pub transaction_date: Date,
    pub first_name: String,
    pub last_name: String,
    pub office: String,
    pub district: String,
    pub owner: String,
    pub asset_description: String,
    pub asset_type: String,
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub amount: String,
    #[serde(
        rename = "capitalGainsOver200USD",
        skip_serializing_if = "Option::is_none"
    )]
    pub capital_gains_over_200_usd: Option<TitleCaseBoolFlag>,
    pub comment: String,
    pub link: String,
}
