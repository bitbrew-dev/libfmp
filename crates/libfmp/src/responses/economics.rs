//! Response rows returned by economics endpoints.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    query::EconomicIndicator,
    types::{ApiDateTime, Change, CountryCode, CurrencyCode, Date, Percentage},
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// Treasury rates across all twelve documented maturities for one date.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct TreasuryRate {
    pub date: Date,
    #[serde(deserialize_with = "required_option")]
    pub month_1: Option<Percentage>,
    #[serde(deserialize_with = "required_option")]
    pub month_2: Option<Percentage>,
    #[serde(deserialize_with = "required_option")]
    pub month_3: Option<Percentage>,
    #[serde(deserialize_with = "required_option")]
    pub month_6: Option<Percentage>,
    #[serde(deserialize_with = "required_option")]
    pub year_1: Option<Percentage>,
    #[serde(deserialize_with = "required_option")]
    pub year_2: Option<Percentage>,
    #[serde(deserialize_with = "required_option")]
    pub year_3: Option<Percentage>,
    #[serde(deserialize_with = "required_option")]
    pub year_5: Option<Percentage>,
    #[serde(deserialize_with = "required_option")]
    pub year_7: Option<Percentage>,
    #[serde(deserialize_with = "required_option")]
    pub year_10: Option<Percentage>,
    #[serde(deserialize_with = "required_option")]
    pub year_20: Option<Percentage>,
    #[serde(deserialize_with = "required_option")]
    pub year_30: Option<Percentage>,
}

/// One observation of a documented or validated open economic indicator.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EconomicIndicatorObservation {
    pub name: EconomicIndicator,
    pub date: Date,
    #[serde(deserialize_with = "required_option")]
    pub value: Option<f64>,
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
    #[serde(deserialize_with = "required_option")]
    pub previous: Option<f64>,
    #[serde(deserialize_with = "required_option")]
    pub estimate: Option<f64>,
    #[serde(deserialize_with = "required_option")]
    pub actual: Option<f64>,
    #[serde(deserialize_with = "required_option")]
    pub change: Option<Change>,
    pub impact: String,
    #[serde(deserialize_with = "required_option")]
    pub change_percentage: Option<Percentage>,
    #[serde(deserialize_with = "required_option")]
    pub unit: Option<String>,
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
