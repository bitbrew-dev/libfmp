//! Financial-report discovery and dynamic JSON response models.

use serde::{Deserialize, Deserializer, Serialize, de};

use crate::{
    codecs::{DynamicObject, FiscalYearString},
    error::SecretUrl,
    query::FiscalPeriod,
    types::{CalendarYear, Ticker},
};

/// One available financial-report period and its protected download links.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportDate {
    pub symbol: Ticker,
    pub fiscal_year: CalendarYear,
    pub period: FiscalPeriod,
    pub link_json: SecretUrl,
    pub link_xlsx: SecretUrl,
}

/// One dynamic financial report with strict identifying headers.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FinancialReportJson {
    pub symbol: Ticker,
    pub period: FiscalPeriod,
    pub year: FiscalYearString,
    #[serde(flatten)]
    pub sections: DynamicObject,
}

impl<'de> Deserialize<'de> for FinancialReportJson {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut sections = DynamicObject::deserialize(deserializer)?;
        Ok(Self {
            symbol: take_header(&mut sections, "symbol")?,
            period: take_header(&mut sections, "period")?,
            year: take_header(&mut sections, "year")?,
            sections,
        })
    }
}

fn take_header<T, E>(sections: &mut DynamicObject, name: &'static str) -> Result<T, E>
where
    T: serde::de::DeserializeOwned,
    E: de::Error,
{
    let value = sections
        .remove(name)
        .ok_or_else(|| E::missing_field(name))?;
    serde_json::from_value(value).map_err(E::custom)
}
