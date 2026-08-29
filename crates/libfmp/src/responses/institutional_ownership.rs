//! Response rows returned by institutional-ownership filing endpoints.

use serde::{Deserialize, Serialize};

use crate::types::{ApiDateTime, CalendarQuarter, CalendarYear, Cik, Cusip, Date, Price, Ticker};

/// One recent institutional-ownership filing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstitutionalHolding {
    pub date: Date,
    pub filing_date: Date,
    pub accepted_date: Date,
    pub cik: Cik,
    pub security_cusip: Cusip,
    pub symbol: Ticker,
    pub name_of_issuer: String,
    pub shares: u64,
    pub title_of_class: String,
    pub shares_type: String,
    pub put_call_share: String,
    pub value: u64,
    pub link: String,
    pub final_link: String,
}

/// One available Form 13F reporting date and its numeric calendar period.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Form13fFilingDate {
    pub date: Date,
    pub year: CalendarYear,
    pub quarter: CalendarQuarter,
}

/// One holder's analytical position in a security for a filing period.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub market_value: u64,
    pub last_market_value: u64,
    pub change_in_market_value: i64,
    pub change_in_market_value_percentage: f64,
    pub shares_number: u64,
    pub last_shares_number: u64,
    pub change_in_shares_number: i64,
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
    pub performance: i64,
    pub performance_percentage: f64,
    pub last_performance: i64,
    pub change_in_performance: i64,
    pub is_counted_for_performance: bool,
}

/// One institutional holder's portfolio and benchmark performance summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HolderPerformanceSummary {
    pub date: Date,
    pub cik: Cik,
    pub investor_name: String,
    pub portfolio_size: u64,
    pub securities_added: u64,
    pub securities_removed: u64,
    pub market_value: u64,
    pub previous_market_value: u64,
    pub change_in_market_value: i64,
    pub change_in_market_value_percentage: f64,
    pub average_holding_period: u64,
    pub average_holding_period_top10: u64,
    pub average_holding_period_top20: u64,
    pub turnover: f64,
    pub turnover_alternate_sell: f64,
    pub turnover_alternate_buy: f64,
    pub performance: i64,
    pub performance_percentage: f64,
    pub last_performance: i64,
    pub change_in_performance: i64,
    #[serde(rename = "performance1year")]
    pub performance_1_year: i64,
    #[serde(rename = "performancePercentage1year")]
    pub performance_percentage_1_year: f64,
    #[serde(rename = "performance3year")]
    pub performance_3_year: i64,
    #[serde(rename = "performancePercentage3year")]
    pub performance_percentage_3_year: f64,
    #[serde(rename = "performance5year")]
    pub performance_5_year: i64,
    #[serde(rename = "performancePercentage5year")]
    pub performance_percentage_5_year: f64,
    pub performance_since_inception: i64,
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
    pub performance: i64,
    pub performance_percentage: f64,
    pub last_performance: i64,
    pub change_in_performance: i64,
}
