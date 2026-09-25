//! Response rows returned by Commitment of Traders endpoints.

use serde::{Deserialize, Serialize};
use serde_json::Number;

use crate::types::{ApiDateTime, Sector, Ticker};

/// One provider-listed Commitment of Traders report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CotReportListing {
    pub symbol: Ticker,
    pub name: String,
}

/// Provider-derived market positioning and sentiment for one contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CotAnalysis {
    pub symbol: Ticker,
    pub date: ApiDateTime,
    pub name: String,
    pub sector: Sector,
    pub exchange: String,
    pub current_long_market_situation: f64,
    pub current_short_market_situation: f64,
    pub market_situation: String,
    pub previous_long_market_situation: f64,
    pub previous_short_market_situation: f64,
    pub previous_market_situation: String,
    #[serde(rename = "netPostion")]
    pub net_position: i64,
    pub previous_net_position: i64,
    pub change_in_net_position: f64,
    pub market_sentiment: String,
    pub reversal_trend: bool,
}

/// A complete, lossless provider Commitment of Traders report row.
///
/// Percentage and concentration values use `Number` because the provider
/// distinguishes integer JSON spellings such as `100` from decimal spellings
/// such as `20.6` in the same response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct CotReport {
    pub symbol: Ticker,
    pub date: ApiDateTime,
    pub name: String,
    pub sector: Sector,
    pub market_and_exchange_names: String,
    pub cftc_contract_market_code: String,
    pub cftc_market_code: String,
    pub cftc_region_code: String,
    pub cftc_commodity_code: String,

    pub open_interest_all: u64,
    pub noncomm_positions_long_all: u64,
    pub noncomm_positions_short_all: u64,
    pub noncomm_positions_spread_all: u64,
    pub comm_positions_long_all: u64,
    pub comm_positions_short_all: u64,
    pub tot_rept_positions_long_all: u64,
    pub tot_rept_positions_short_all: u64,
    pub nonrept_positions_long_all: u64,
    pub nonrept_positions_short_all: u64,

    pub open_interest_old: u64,
    pub noncomm_positions_long_old: u64,
    pub noncomm_positions_short_old: u64,
    pub noncomm_positions_spread_old: u64,
    pub comm_positions_long_old: u64,
    pub comm_positions_short_old: u64,
    pub tot_rept_positions_long_old: u64,
    pub tot_rept_positions_short_old: u64,
    pub nonrept_positions_long_old: u64,
    pub nonrept_positions_short_old: u64,

    pub open_interest_other: u64,
    pub noncomm_positions_long_other: u64,
    pub noncomm_positions_short_other: u64,
    pub noncomm_positions_spread_other: u64,
    pub comm_positions_long_other: u64,
    pub comm_positions_short_other: u64,
    pub tot_rept_positions_long_other: u64,
    pub tot_rept_positions_short_other: u64,
    pub nonrept_positions_long_other: u64,
    pub nonrept_positions_short_other: u64,

    pub change_in_open_interest_all: i64,
    pub change_in_noncomm_long_all: i64,
    pub change_in_noncomm_short_all: i64,
    #[serde(rename = "changeInNoncommSpeadAll")]
    pub change_in_noncomm_spread_all: i64,
    pub change_in_comm_long_all: i64,
    pub change_in_comm_short_all: i64,
    pub change_in_tot_rept_long_all: i64,
    pub change_in_tot_rept_short_all: i64,
    pub change_in_nonrept_long_all: i64,
    pub change_in_nonrept_short_all: i64,

    pub pct_of_open_interest_all: Number,
    pub pct_of_oi_noncomm_long_all: Number,
    pub pct_of_oi_noncomm_short_all: Number,
    pub pct_of_oi_noncomm_spread_all: Number,
    pub pct_of_oi_comm_long_all: Number,
    pub pct_of_oi_comm_short_all: Number,
    pub pct_of_oi_tot_rept_long_all: Number,
    pub pct_of_oi_tot_rept_short_all: Number,
    pub pct_of_oi_nonrept_long_all: Number,
    pub pct_of_oi_nonrept_short_all: Number,

    #[serde(rename = "pctOfOpenInterestOl")]
    pub pct_of_open_interest_old: Number,
    #[serde(rename = "pctOfOiNoncommLongOl")]
    pub pct_of_oi_noncomm_long_old: Number,
    #[serde(rename = "pctOfOiNoncommShortOl")]
    pub pct_of_oi_noncomm_short_old: Number,
    #[serde(rename = "pctOfOiNoncommSpreadOl")]
    pub pct_of_oi_noncomm_spread_old: Number,
    #[serde(rename = "pctOfOiCommLongOl")]
    pub pct_of_oi_comm_long_old: Number,
    #[serde(rename = "pctOfOiCommShortOl")]
    pub pct_of_oi_comm_short_old: Number,
    #[serde(rename = "pctOfOiTotReptLongOl")]
    pub pct_of_oi_tot_rept_long_old: Number,
    #[serde(rename = "pctOfOiTotReptShortOl")]
    pub pct_of_oi_tot_rept_short_old: Number,
    #[serde(rename = "pctOfOiNonreptLongOl")]
    pub pct_of_oi_nonrept_long_old: Number,
    #[serde(rename = "pctOfOiNonreptShortOl")]
    pub pct_of_oi_nonrept_short_old: Number,

    pub pct_of_open_interest_other: Number,
    pub pct_of_oi_noncomm_long_other: Number,
    pub pct_of_oi_noncomm_short_other: Number,
    pub pct_of_oi_noncomm_spread_other: Number,
    pub pct_of_oi_comm_long_other: Number,
    pub pct_of_oi_comm_short_other: Number,
    pub pct_of_oi_tot_rept_long_other: Number,
    pub pct_of_oi_tot_rept_short_other: Number,
    pub pct_of_oi_nonrept_long_other: Number,
    pub pct_of_oi_nonrept_short_other: Number,

    pub traders_tot_all: u64,
    pub traders_noncomm_long_all: u64,
    pub traders_noncomm_short_all: u64,
    pub traders_noncomm_spread_all: u64,
    pub traders_comm_long_all: u64,
    pub traders_comm_short_all: u64,
    pub traders_tot_rept_long_all: u64,
    pub traders_tot_rept_short_all: u64,

    #[serde(rename = "tradersTotOl")]
    pub traders_tot_old: u64,
    #[serde(rename = "tradersNoncommLongOl")]
    pub traders_noncomm_long_old: u64,
    #[serde(rename = "tradersNoncommShortOl")]
    pub traders_noncomm_short_old: u64,
    #[serde(rename = "tradersNoncommSpeadOl")]
    pub traders_noncomm_spread_old: u64,
    #[serde(rename = "tradersCommLongOl")]
    pub traders_comm_long_old: u64,
    #[serde(rename = "tradersCommShortOl")]
    pub traders_comm_short_old: u64,
    #[serde(rename = "tradersTotReptLongOl")]
    pub traders_tot_rept_long_old: u64,
    #[serde(rename = "tradersTotReptShortOl")]
    pub traders_tot_rept_short_old: u64,

    pub traders_tot_other: u64,
    pub traders_noncomm_long_other: u64,
    pub traders_noncomm_short_other: u64,
    pub traders_noncomm_spread_other: u64,
    pub traders_comm_long_other: u64,
    pub traders_comm_short_other: u64,
    pub traders_tot_rept_long_other: u64,
    pub traders_tot_rept_short_other: u64,

    pub conc_gross_le4_tdr_long_all: Number,
    pub conc_gross_le4_tdr_short_all: Number,
    pub conc_gross_le8_tdr_long_all: Number,
    pub conc_gross_le8_tdr_short_all: Number,
    pub conc_net_le4_tdr_long_all: Number,
    pub conc_net_le4_tdr_short_all: Number,
    pub conc_net_le8_tdr_long_all: Number,
    pub conc_net_le8_tdr_short_all: Number,

    #[serde(rename = "concGrossLe4TdrLongOl")]
    pub conc_gross_le4_tdr_long_old: Number,
    #[serde(rename = "concGrossLe4TdrShortOl")]
    pub conc_gross_le4_tdr_short_old: Number,
    #[serde(rename = "concGrossLe8TdrLongOl")]
    pub conc_gross_le8_tdr_long_old: Number,
    #[serde(rename = "concGrossLe8TdrShortOl")]
    pub conc_gross_le8_tdr_short_old: Number,
    #[serde(rename = "concNetLe4TdrLongOl")]
    pub conc_net_le4_tdr_long_old: Number,
    #[serde(rename = "concNetLe4TdrShortOl")]
    pub conc_net_le4_tdr_short_old: Number,
    #[serde(rename = "concNetLe8TdrLongOl")]
    pub conc_net_le8_tdr_long_old: Number,
    #[serde(rename = "concNetLe8TdrShortOl")]
    pub conc_net_le8_tdr_short_old: Number,

    pub conc_gross_le4_tdr_long_other: Number,
    pub conc_gross_le4_tdr_short_other: Number,
    pub conc_gross_le8_tdr_long_other: Number,
    pub conc_gross_le8_tdr_short_other: Number,
    pub conc_net_le4_tdr_long_other: Number,
    pub conc_net_le4_tdr_short_other: Number,
    pub conc_net_le8_tdr_long_other: Number,
    pub conc_net_le8_tdr_short_other: Number,

    pub contract_units: String,
}
