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

/// One current or historical profile for a member of Congress.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CongressionalMemberProfile {
    #[serde(rename = "senateID")]
    pub member_id: CongressionalMemberId,
    pub first_name: String,
    pub last_name: String,
    pub birth_date: Date,
    pub latest_party: String,
    pub latest_state: String,
    pub latest_position: String,
    pub image: String,
    pub active: bool,
    pub years_active: f64,
}

/// One congressional position held by a member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CongressionalMemberPosition {
    #[serde(rename = "senateID")]
    pub member_id: CongressionalMemberId,
    pub congress_number: u32,
    pub start_date: Date,
    pub end_date: Option<Date>,
    pub party: String,
    pub position: String,
    pub state: String,
    pub years_in_term: f64,
}
