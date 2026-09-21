//! Response models for SEC filings, company lookup, profiles, and SIC data.
//!
//! The provider documents every SEC-filings route as available for
//! US-based companies only. The Python binding exposes these models under
//! `fmp.sec_filings`.

use serde::{Deserialize, Serialize};

use crate::{
    codecs::{DynamicJson, NumericString},
    types::{
        ApiDateTime, Cik, CountryCode, CurrencyCode, Date, ExchangeCode, FormType, Isin, Sector,
        Ticker,
    },
};

/// One SEC filing row shared by all five filing feeds and searches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecFiling {
    pub symbol: Ticker,
    pub cik: Cik,
    pub filing_date: ApiDateTime,
    pub accepted_date: ApiDateTime,
    pub form_type: FormType,
    #[serde(default)]
    pub has_financials: Option<bool>,
    pub link: String,
    pub final_link: String,
}

/// One company identity returned by the three SEC company-search routes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecCompanySearchResult {
    pub symbol: Ticker,
    pub name: String,
    pub cik: Cik,
    pub sic_code: String,
    pub industry_title: String,
    pub business_address: String,
    pub phone_number: String,
}

/// One full SEC company profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecCompanyProfile {
    pub symbol: Ticker,
    pub cik: Cik,
    pub registrant_name: String,
    pub sic_code: String,
    pub sic_description: String,
    pub sic_group: String,
    pub isin: Isin,
    pub business_address: String,
    pub mailing_address: String,
    pub phone_number: String,
    pub postal_code: String,
    pub city: String,
    pub state: String,
    pub country: CountryCode,
    pub description: String,
    pub ceo: String,
    pub website: String,
    pub exchange: ExchangeCode,
    pub state_location: String,
    pub state_of_incorporation: String,
    pub fiscal_year_end: String,
    pub ipo_date: Date,
    pub employees: NumericString,
    pub sec_filings_url: String,
    pub tax_identification_number: String,
    pub fifty_two_week_range: String,
    pub is_active: bool,
    pub asset_type: String,
    pub open_figi_composite: String,
    pub price_currency: CurrencyCode,
    pub market_sector: Sector,
    #[serde(deserialize_with = "deserialize_nullable")]
    pub security_type: Option<DynamicJson>,
    pub is_etf: bool,
    pub is_adr: bool,
    pub is_fund: bool,
}

fn deserialize_nullable<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One Standard Industrial Classification directory row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SicClassification {
    pub office: String,
    pub sic_code: String,
    pub industry_title: String,
}
