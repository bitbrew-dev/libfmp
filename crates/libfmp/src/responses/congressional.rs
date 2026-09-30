//! Response rows returned by congressional financial-disclosure endpoints.
//!
//! The provider documents every congressional route as available for
//! US-based companies only. The Python binding exposes these models under
//! `fmp.congressional`.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    codecs::{DynamicJson, OpaqueDateText, TitleCaseBoolFlag},
    types::{CalendarYear, CongressionalMemberId, Date, FormType},
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One financial trade disclosed by a member of Congress.
///
/// The provider calls the cross-chamber member identifier `senateID`, even for
/// House members. `symbol` deliberately remains a string because the
/// documented Senate-by-name response contains an empty value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
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
#[non_exhaustive]
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
    #[serde(deserialize_with = "required_option")]
    pub image: Option<String>,
    pub active: bool,
    pub years_active: f64,
}

/// One congressional position held by a member.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CongressionalMemberPosition {
    #[serde(rename = "senateID")]
    pub member_id: CongressionalMemberId,
    pub congress_number: u32,
    pub start_date: Date,
    #[serde(deserialize_with = "required_option")]
    pub end_date: Option<Date>,
    pub party: String,
    pub position: String,
    pub state: String,
    pub years_in_term: f64,
}

/// A minimum and maximum value disclosed for a congressional asset or income.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CongressionalNetWorthRange {
    pub min: i64,
    pub max: i64,
}

/// Opaque provider details for a disclosed debt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CongressionalDebtDetails {
    pub date_incurred: OpaqueDateText,
}

/// One itemized congressional net-worth disclosure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CongressionalMemberNetWorth {
    #[serde(rename = "senateID")]
    pub member_id: CongressionalMemberId,
    pub form_type: FormType,
    pub year: CalendarYear,
    pub filing_date: Date,
    pub section: String,
    pub category: String,
    pub name: String,
    pub asset_type: String,
    #[serde(deserialize_with = "required_option")]
    pub income_type: Option<String>,
    pub owner: String,
    #[serde(deserialize_with = "required_option")]
    pub comment: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub debt_details: Option<CongressionalDebtDetails>,
    #[serde(deserialize_with = "required_option")]
    pub value_range: Option<CongressionalNetWorthRange>,
    pub value: i64,
    #[serde(deserialize_with = "required_option")]
    pub income_range: Option<CongressionalNetWorthRange>,
    #[serde(deserialize_with = "required_option")]
    pub income: Option<DynamicJson>,
    pub link: String,
}

/// Aggregated congressional net-worth totals for one filing year.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CongressionalMemberNetWorthAggregate {
    #[serde(rename = "senateID")]
    pub member_id: CongressionalMemberId,
    pub year: CalendarYear,
    pub total: i64,
    pub real_estate_liabilities: i64,
    pub cash_and_cash_equivalents: i64,
    pub business_and_self_employment: i64,
    pub real_estate: i64,
    pub ownership_interest: i64,
    pub stock: i64,
    pub options: i64,
    pub revolving_and_credit_lines: i64,
    pub asset_backed_securities: i64,
    pub business_liabilities: i64,
    #[serde(rename = "mutualFundsAndETFs")]
    pub mutual_funds_and_etfs: i64,
}
