//! Response rows returned by environmental, social, and governance endpoints.
//!
//! The Python binding exposes these models under `fmp.esg`.

use serde::{Deserialize, Deserializer, Serialize};

use crate::types::{CalendarYear, Cik, Date, Industry, Sector, Ticker};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One company ESG disclosure filing and its component scores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EsgDisclosure {
    pub date: Date,
    pub accepted_date: Date,
    pub symbol: Ticker,
    pub cik: Cik,
    #[serde(deserialize_with = "required_option")]
    pub company_name: Option<String>,
    #[serde(deserialize_with = "required_option")]
    pub form_type: Option<crate::types::FormType>,
    pub environmental_score: f64,
    pub social_score: f64,
    pub governance_score: f64,
    #[serde(rename = "ESGScore")]
    pub esg_score: f64,
    pub url: String,
}

/// One company's ESG risk rating for a fiscal year.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EsgRating {
    pub symbol: Ticker,
    pub cik: Cik,
    #[serde(deserialize_with = "required_option")]
    pub company_name: Option<String>,
    pub industry: Industry,
    pub fiscal_year: CalendarYear,
    #[serde(rename = "ESGRiskRating")]
    pub esg_risk_rating: String,
    pub industry_rank: String,
}

/// One sector ESG benchmark for a fiscal year.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EsgBenchmark {
    pub fiscal_year: CalendarYear,
    pub sector: Sector,
    pub environmental_score: f64,
    pub social_score: f64,
    pub governance_score: f64,
    #[serde(rename = "ESGScore")]
    pub esg_score: f64,
}
