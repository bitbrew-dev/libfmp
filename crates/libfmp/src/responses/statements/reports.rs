//! Financial-report discovery and dynamic JSON response models.

use serde::{Deserialize, Deserializer, Serialize, Serializer, de, ser::SerializeMap};

use crate::{
    codecs::{DynamicObject, FiscalYearString},
    error::SecretUrl,
    query::FiscalPeriod,
    types::{CalendarYear, Ticker},
};

/// One available financial-report period and its protected download links.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct FinancialReportDate {
    pub symbol: Ticker,
    pub fiscal_year: CalendarYear,
    pub period: FiscalPeriod,
    pub link_json: SecretUrl,
    pub link_xlsx: SecretUrl,
}

/// One dynamic financial report with strict identifying headers.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct FinancialReportJson {
    pub symbol: Ticker,
    pub period: FiscalPeriod,
    pub year: FiscalYearString,
    pub sections: DynamicObject,
}

impl Serialize for FinancialReportJson {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        for reserved in ["symbol", "period", "year"] {
            if self.sections.contains_key(reserved) {
                return Err(serde::ser::Error::custom(format_args!(
                    "financial report section key `{reserved}` is reserved"
                )));
            }
        }

        let mut map = serializer.serialize_map(Some(self.sections.len() + 3))?;
        map.serialize_entry("symbol", &self.symbol)?;
        map.serialize_entry("period", &self.period)?;
        map.serialize_entry("year", &self.year)?;
        for (name, value) in &self.sections {
            map.serialize_entry(name, value)?;
        }
        map.end()
    }
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
