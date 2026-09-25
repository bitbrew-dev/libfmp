//! Response rows returned by institutional-ownership filing endpoints.

use serde::{Deserialize, Serialize};

use crate::types::{
    ApiDateTime, CalendarQuarter, CalendarYear, Cik, Cusip, Date, MarketValue, Price, Quantity,
    Ticker,
};

/// One recent institutional-ownership filing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct InstitutionalOwnershipFiling {
    pub cik: Cik,
    pub name: String,
    pub date: Date,
    pub filing_date: ApiDateTime,
    pub accepted_date: ApiDateTime,
    pub form_type: String,
    pub link: String,
    pub final_link: String,
}

/// One security position extracted from an institutional-ownership filing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct InstitutionalHolding {
    pub date: Date,
    pub filing_date: Date,
    pub accepted_date: Date,
    pub cik: Cik,
    pub security_cusip: Cusip,
    pub symbol: Ticker,
    pub name_of_issuer: String,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub shares: Quantity,
    pub title_of_class: String,
    pub shares_type: String,
    pub put_call_share: String,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub value: MarketValue,
    pub link: String,
    pub final_link: String,
}

/// One available Form 13F reporting date and its numeric calendar period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct Form13fFilingDate {
    pub date: Date,
    pub year: CalendarYear,
    pub quarter: CalendarQuarter,
}

/// One holder's analytical position in a security for a filing period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct InstitutionalHolderAnalytics {
    pub date: Date,
    pub cik: Cik,
    pub filing_date: Date,
    pub investor_name: String,
    pub symbol: Ticker,
    pub security_name: String,
    pub type_of_security: String,
    pub security_cusip: Cusip,
    pub shares_type: String,
    pub put_call_share: String,
    pub investment_discretion: String,
    pub industry_title: String,
    pub weight: f64,
    pub last_weight: f64,
    pub change_in_weight: f64,
    pub change_in_weight_percentage: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_value: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub last_market_value: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub change_in_market_value: MarketValue,
    pub change_in_market_value_percentage: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub shares_number: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub last_shares_number: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub change_in_shares_number: Quantity,
    pub change_in_shares_number_percentage: f64,
    pub quarter_end_price: Price,
    pub avg_price_paid: Price,
    pub is_new: bool,
    pub is_sold_out: bool,
    pub ownership: f64,
    pub last_ownership: f64,
    pub change_in_ownership: f64,
    pub change_in_ownership_percentage: f64,
    pub holding_period: u64,
    pub first_added: Date,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub performance: MarketValue,
    pub performance_percentage: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub last_performance: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub change_in_performance: MarketValue,
    pub is_counted_for_performance: bool,
}

/// One institutional holder's portfolio and benchmark performance summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct HolderPerformanceSummary {
    pub date: Date,
    pub cik: Cik,
    pub investor_name: String,
    pub portfolio_size: u64,
    pub securities_added: u64,
    pub securities_removed: u64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_value: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub previous_market_value: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub change_in_market_value: MarketValue,
    pub change_in_market_value_percentage: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub average_holding_period: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub average_holding_period_top10: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub average_holding_period_top20: f64,
    pub turnover: f64,
    pub turnover_alternate_sell: f64,
    pub turnover_alternate_buy: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub performance: MarketValue,
    pub performance_percentage: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub last_performance: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub change_in_performance: MarketValue,
    #[serde(rename = "performance1year")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub performance_1_year: MarketValue,
    #[serde(rename = "performancePercentage1year")]
    pub performance_percentage_1_year: f64,
    #[serde(rename = "performance3year")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub performance_3_year: MarketValue,
    #[serde(rename = "performancePercentage3year")]
    pub performance_percentage_3_year: f64,
    #[serde(rename = "performance5year")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub performance_5_year: MarketValue,
    #[serde(rename = "performancePercentage5year")]
    pub performance_percentage_5_year: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub performance_since_inception: MarketValue,
    pub performance_since_inception_percentage: f64,
    #[serde(rename = "performanceRelativeToSP500Percentage")]
    pub performance_relative_to_sp500_percentage: f64,
    #[serde(rename = "performance1yearRelativeToSP500Percentage")]
    pub performance_1_year_relative_to_sp500_percentage: f64,
    #[serde(rename = "performance3yearRelativeToSP500Percentage")]
    pub performance_3_year_relative_to_sp500_percentage: f64,
    #[serde(rename = "performance5yearRelativeToSP500Percentage")]
    pub performance_5_year_relative_to_sp500_percentage: f64,
    #[serde(rename = "performanceSinceInceptionRelativeToSP500Percentage")]
    pub performance_since_inception_relative_to_sp500_percentage: f64,
}

/// One industry allocation and performance row for an institutional holder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct HolderIndustryBreakdown {
    pub date: Date,
    pub cik: Cik,
    pub investor_name: String,
    pub industry_title: String,
    pub weight: f64,
    pub last_weight: f64,
    pub change_in_weight: f64,
    pub change_in_weight_percentage: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub performance: MarketValue,
    pub performance_percentage: f64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub last_performance: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub change_in_performance: MarketValue,
}

/// Cross-holder position totals and changes for one security and filing period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct InstitutionalPositionSummary {
    pub symbol: Ticker,
    pub cik: Cik,
    pub date: Date,
    pub investors_holding: u64,
    pub last_investors_holding: u64,
    pub investors_holding_change: i64,
    #[serde(rename = "numberOf13Fshares")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub number_of_13f_shares: Quantity,
    #[serde(rename = "lastNumberOf13Fshares")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub last_number_of_13f_shares: Quantity,
    #[serde(rename = "numberOf13FsharesChange")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub number_of_13f_shares_change: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_invested: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub last_total_invested: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_invested_change: MarketValue,
    pub ownership_percent: f64,
    pub last_ownership_percent: f64,
    pub ownership_percent_change: f64,
    pub new_positions: u64,
    pub last_new_positions: u64,
    pub new_positions_change: i64,
    pub increased_positions: u64,
    pub last_increased_positions: u64,
    pub increased_positions_change: i64,
    pub closed_positions: u64,
    pub last_closed_positions: u64,
    pub closed_positions_change: i64,
    pub reduced_positions: u64,
    pub last_reduced_positions: u64,
    pub reduced_positions_change: i64,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_calls: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub last_total_calls: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_calls_change: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_puts: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub last_total_puts: Quantity,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub total_puts_change: Quantity,
    pub put_call_ratio: f64,
    pub last_put_call_ratio: f64,
    pub put_call_ratio_change: f64,
}

/// Aggregate value for one US industry and filing date.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct InstitutionalIndustrySummary {
    pub industry_title: String,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub industry_value: MarketValue,
    pub date: Date,
}
