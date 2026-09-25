//! Response rows returned by calendar endpoints.

use serde::{Deserialize, Deserializer, Serialize};

use crate::{
    codecs::{DynamicJson, IsoTimestamp, empty_or_null_date},
    types::{Cik, Date, ExchangeCode, MarketValue, Percentage, Price, SplitTerm, Ticker},
};

fn required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

/// One company or market-wide dividend event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct DividendEvent {
    pub symbol: Ticker,
    pub date: Date,
    pub record_date: Date,
    pub payment_date: Date,
    #[serde(with = "empty_or_null_date")]
    pub declaration_date: Option<Date>,
    pub adj_dividend: Price,
    pub dividend: Price,
    pub r#yield: Percentage,
    pub frequency: String,
}

/// One company or market-wide earnings event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct EarningsEvent {
    pub symbol: Ticker,
    pub date: Date,
    #[serde(deserialize_with = "required_option")]
    pub eps_actual: Option<f64>,
    pub eps_estimated: f64,
    #[serde(deserialize_with = "required_option")]
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize_option")]
    pub revenue_actual: Option<MarketValue>,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub revenue_estimated: MarketValue,
    pub last_updated: Date,
}

/// One worldwide IPO calendar event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct IpoCalendarEvent {
    pub symbol: Ticker,
    pub date: Date,
    pub daa: IsoTimestamp,
    pub company: String,
    pub exchange: ExchangeCode,
    pub actions: String,
    #[serde(deserialize_with = "required_option")]
    pub shares: Option<DynamicJson>,
    #[serde(deserialize_with = "required_option")]
    pub price_range: Option<DynamicJson>,
    #[serde(deserialize_with = "required_option")]
    pub market_cap: Option<DynamicJson>,
}

/// One US IPO disclosure filing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct IpoDisclosure {
    pub symbol: Ticker,
    pub filing_date: Date,
    pub accepted_date: Date,
    pub effectiveness_date: Date,
    pub cik: Cik,
    pub form: String,
    pub url: String,
}

/// One US IPO prospectus filing and its documented offering values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct IpoProspectus {
    pub symbol: Ticker,
    pub accepted_date: Date,
    pub filing_date: Date,
    pub ipo_date: Date,
    pub cik: Cik,
    pub price_public_per_share: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub price_public_total: MarketValue,
    pub discounts_and_commissions_per_share: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub discounts_and_commissions_total: MarketValue,
    pub proceeds_before_expenses_per_share: Price,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub proceeds_before_expenses_total: MarketValue,
    pub form: String,
    pub url: String,
}

/// One company or market-wide stock-split event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct StockSplitEvent {
    pub symbol: Ticker,
    pub date: Date,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub numerator: SplitTerm,
    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]
    pub denominator: SplitTerm,
    pub split_type: String,
}
