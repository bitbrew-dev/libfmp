//! Response rows returned by environmental, social, and governance endpoints.
//!
//! Future Python bindings reserve these models under `fmp.esg`.

use serde::{Deserialize, Serialize};

use crate::types::{CalendarYear, Cik, Date, Industry, Sector, Ticker};

/// One company ESG disclosure filing and its component scores.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EsgDisclosure {
    pub date: Date,
    pub accepted_date: Date,
    pub symbol: Ticker,
    pub cik: Cik,
    pub company_name: String,
    pub form_type: crate::types::FormType,
    pub environmental_score: f64,
    pub social_score: f64,
    pub governance_score: f64,
    #[serde(rename = "ESGScore")]
    pub esg_score: f64,
    pub url: String,
}

/// One company's ESG risk rating for a fiscal year.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EsgRating {
    pub symbol: Ticker,
    pub cik: Cik,
    pub company_name: String,
    pub industry: Industry,
    pub fiscal_year: CalendarYear,
    #[serde(rename = "ESGRiskRating")]
    pub esg_risk_rating: String,
    pub industry_rank: String,
}

/// One sector ESG benchmark for a fiscal year.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
