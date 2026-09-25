//! Response rows returned by economics endpoints.

use serde::{Deserialize, Serialize};

use crate::{
    query::EconomicIndicator,
    types::{ApiDateTime, Change, CountryCode, CurrencyCode, Date, Percentage},
};

/// Treasury rates across all twelve documented maturities for one date.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct TreasuryRate {
    pub date: Date,
    pub month_1: Percentage,
    pub month_2: Percentage,
    pub month_3: Percentage,
    pub month_6: Percentage,
    pub year_1: Percentage,
    pub year_2: Percentage,
    pub year_3: Percentage,
    pub year_5: Percentage,
    pub year_7: Percentage,
    pub year_10: Percentage,
    pub year_20: Percentage,
    pub year_30: Percentage,
}

/// One observation of a documented or validated open economic indicator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EconomicIndicatorObservation {
    pub name: EconomicIndicator,
    pub date: Date,
    pub value: f64,
}

/// One scheduled economic data release.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EconomicCalendarEvent {
    pub date: ApiDateTime,
    pub country: CountryCode,
    pub event: String,
    pub currency: CurrencyCode,
    pub previous: f64,
    pub estimate: f64,
    pub actual: f64,
    pub change: Change,
    pub impact: String,
    pub change_percentage: Percentage,
    pub unit: String,
}

/// One country's documented market and total-equity risk premiums.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct MarketRiskPremium {
    /// Full country name, not a provider country code.
    pub country: String,
    pub continent: String,
    pub country_risk_premium: Percentage,
    pub total_equity_risk_premium: Percentage,
}
