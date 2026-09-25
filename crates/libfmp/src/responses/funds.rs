//! Response rows returned by ETF and mutual-fund endpoints.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::{IsoTimestamp, NumericString, PercentString, YnFlag},
    types::{
        ApiDateTime, Cik, Count, CountryCode, CurrencyCode, Cusip, Date, Industry, Isin, Lei,
        MarketValue, Percentage, Quantity, Sector, Ticker, Volume,
    },
};

/// One asset held by an ETF or mutual fund.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EtfFundHolding {
    pub symbol: Ticker,
    pub asset: Ticker,
    pub name: String,
    pub isin: Isin,
    pub security_cusip: Cusip,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub shares_number: Quantity,
    pub weight_percentage: Percentage,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_value: MarketValue,
    pub updated_at: ApiDateTime,
}

/// One sector exposure nested within a fund-information row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EtfSectorExposure {
    pub industry: Industry,
    pub exposure: Percentage,
}

/// Descriptive, structural, and trading information for an ETF or mutual fund.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EtfFundInfo {
    pub symbol: Ticker,
    pub name: String,
    pub description: String,
    pub isin: Isin,
    pub asset_class: String,
    pub security_cusip: Cusip,
    pub domicile: CountryCode,
    pub website: String,
    pub etf_company: String,
    pub expense_ratio: Percentage,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub assets_under_management: MarketValue,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub avg_volume: Volume,
    pub inception_date: Date,
    pub nav: f64,
    pub nav_currency: CurrencyCode,
    pub holdings_count: Count,
    pub is_actively_trading: bool,
    pub updated_at: IsoTimestamp,
    pub sectors_list: Vec<EtfSectorExposure>,
}

/// One country allocation reported as an exact percent-bearing string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EtfCountryWeighting {
    pub country: String,
    pub weight_percentage: PercentString,
}

/// One ETF's exposure to a requested asset.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EtfAssetExposure {
    pub symbol: Ticker,
    pub asset: Ticker,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub shares_number: Quantity,
    pub weight_percentage: Percentage,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub market_value: MarketValue,
}

/// One numeric sector allocation for an ETF or mutual fund.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EtfSectorWeighting {
    pub symbol: Ticker,
    pub sector: Sector,
    pub weight_percentage: Percentage,
}

/// One fund holder from the latest disclosure for a security.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct FundDisclosureHolder {
    pub cik: Cik,
    pub holder: String,
    pub security_cusip: Cusip,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub shares: Quantity,
    pub date_reported: Date,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub change: Quantity,
    pub weight_percent: Percentage,
}

/// One position in a mutual-fund disclosure filing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct FundDisclosure {
    pub cik: Cik,
    pub date: Date,
    pub accepted_date: ApiDateTime,
    pub symbol: Ticker,
    pub name: String,
    pub lei: Lei,
    pub title: String,
    pub cusip: Cusip,
    pub isin: Isin,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub balance: Quantity,
    pub units: String,
    #[serde(rename = "cur_cd")]
    pub currency_code: CurrencyCode,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub val_usd: MarketValue,
    pub pct_val: Percentage,
    pub payoff_profile: String,
    pub asset_cat: String,
    pub issuer_cat: String,
    pub inv_country: CountryCode,
    pub is_restricted_sec: YnFlag,
    pub fair_val_level: NumericString,
    pub is_cash_collateral: YnFlag,
    pub is_non_cash_collateral: YnFlag,
    pub is_loan_by_fund: YnFlag,
}

/// One result from searching disclosure holders by fund or ETF name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct FundDisclosureSearchResult {
    pub symbol: Ticker,
    pub cik: Cik,
    pub class_id: String,
    pub series_id: String,
    pub entity_name: String,
    pub entity_org_type: NumericString,
    pub series_name: String,
    pub class_name: String,
    pub reporting_file_number: String,
    pub address: String,
    pub city: String,
    pub zip_code: String,
    pub state: String,
}

/// One available fund-disclosure reporting date and calendar period.
///
/// Fund disclosures and Form 13F filings use the same exact three-field wire
/// contract, so this endpoint-facing name reuses that existing response row.
pub type FundDisclosureDate = crate::responses::institutional_ownership::Form13fFilingDate;
