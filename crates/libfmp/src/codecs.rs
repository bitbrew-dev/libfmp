//! Lossless response values for provider fields with mixed wire representations.
//!
//! These types are intentionally narrow. Date-like strings are not passed
//! through one permissive parser because individual endpoint fields document
//! different formats.

use std::{error::Error, fmt, str::FromStr};

use chrono::{DateTime, NaiveDate};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};
use serde_json::Number;

use crate::types::{ApiDateTime, Date, InvalidTemporalValue};

/// A fiscal year that preserves whether the provider sent a JSON integer or string.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum FiscalYear {
    Integer(u32),
    String(FiscalYearString),
}

impl FiscalYear {
    pub const fn as_integer(&self) -> Option<u32> {
        match self {
            Self::Integer(value) => Some(*value),
            Self::String(_) => None,
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            Self::Integer(_) => None,
            Self::String(value) => Some(value.as_str()),
        }
    }
}

/// A fiscal year received as digit-only JSON text.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FiscalYearString(String);

impl FiscalYearString {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidFiscalYear> {
        let value = value.into();
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(InvalidFiscalYear);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for FiscalYearString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for FiscalYearString {
    type Err = InvalidFiscalYear;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

/// Why a fiscal-year string was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidFiscalYear;

impl fmt::Display for InvalidFiscalYear {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("fiscal year must contain only ASCII digits")
    }
}

impl Error for InvalidFiscalYear {}

impl Serialize for FiscalYearString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for FiscalYearString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

impl Serialize for FiscalYear {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Integer(value) => serializer.serialize_u32(*value),
            Self::String(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for FiscalYear {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum WireFiscalYear {
            Integer(u32),
            String(FiscalYearString),
        }

        Ok(match WireFiscalYear::deserialize(deserializer)? {
            WireFiscalYear::Integer(value) => Self::Integer(value),
            WireFiscalYear::String(value) => Self::String(value),
        })
    }
}

/// Why a string-backed numeric representation was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidNumericString;

impl fmt::Display for InvalidNumericString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("value must be a JSON numeric string")
    }
}

impl Error for InvalidNumericString {}

/// A validated JSON number kept in its original string representation.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NumericString(String);

impl NumericString {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidNumericString> {
        let value = value.into();
        if value.trim() != value || serde_json::from_str::<Number>(&value).is_err() {
            return Err(InvalidNumericString);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for NumericString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for NumericString {
    type Err = InvalidNumericString;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl Serialize for NumericString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for NumericString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// A JSON number or numeric string, preserving the provider's wire kind.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum NumberOrNumericString {
    Number(Number),
    String(NumericString),
}

impl NumberOrNumericString {
    pub const fn as_number(&self) -> Option<&Number> {
        match self {
            Self::Number(value) => Some(value),
            Self::String(_) => None,
        }
    }

    pub fn as_numeric_string(&self) -> Option<&str> {
        match self {
            Self::Number(_) => None,
            Self::String(value) => Some(value.as_str()),
        }
    }
}

impl Serialize for NumberOrNumericString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Number(value) => value.serialize(serializer),
            Self::String(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for NumberOrNumericString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum WireNumber {
            Number(Number),
            String(NumericString),
        }

        Ok(match WireNumber::deserialize(deserializer)? {
            WireNumber::Number(value) => Self::Number(value),
            WireNumber::String(value) => Self::String(value),
        })
    }
}

/// A percentage string containing a numeric representation followed by `%`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PercentString(String);

impl PercentString {
    pub fn new(value: impl Into<String>) -> Result<Self, InvalidNumericString> {
        let value = value.into();
        let Some(number) = value.strip_suffix('%') else {
            return Err(InvalidNumericString);
        };
        NumericString::new(number)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PercentString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for PercentString {
    type Err = InvalidNumericString;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl Serialize for PercentString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for PercentString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// A percentage preserving number, numeric-string, and percent-string forms.
///
/// No variant is scaled: `0.25`, `"0.25"`, and `"25%"` remain exactly those
/// three representations.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum PercentageValue {
    Number(Number),
    NumericString(NumericString),
    PercentString(PercentString),
}

impl PercentageValue {
    pub const fn as_number(&self) -> Option<&Number> {
        match self {
            Self::Number(value) => Some(value),
            Self::NumericString(_) | Self::PercentString(_) => None,
        }
    }

    pub fn as_numeric_string(&self) -> Option<&str> {
        match self {
            Self::NumericString(value) => Some(value.as_str()),
            Self::Number(_) | Self::PercentString(_) => None,
        }
    }

    pub fn as_percent_string(&self) -> Option<&str> {
        match self {
            Self::PercentString(value) => Some(value.as_str()),
            Self::Number(_) | Self::NumericString(_) => None,
        }
    }
}

impl Serialize for PercentageValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Number(value) => value.serialize(serializer),
            Self::NumericString(value) => value.serialize(serializer),
            Self::PercentString(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for PercentageValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum WirePercentage {
            Number(Number),
            String(String),
        }

        match WirePercentage::deserialize(deserializer)? {
            WirePercentage::Number(value) => Ok(Self::Number(value)),
            WirePercentage::String(value) if value.ends_with('%') => PercentString::new(value)
                .map(Self::PercentString)
                .map_err(de::Error::custom),
            WirePercentage::String(value) => NumericString::new(value)
                .map(Self::NumericString)
                .map_err(de::Error::custom),
        }
    }
}

macro_rules! string_bool {
    ($name:ident, $true_wire:literal, $false_wire:literal, $description:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name {
            True,
            False,
        }

        impl $name {
            pub const fn as_bool(self) -> bool {
                matches!(self, Self::True)
            }

            pub const fn as_str(self) -> &'static str {
                match self {
                    Self::True => $true_wire,
                    Self::False => $false_wire,
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                match String::deserialize(deserializer)?.as_str() {
                    $true_wire => Ok(Self::True),
                    $false_wire => Ok(Self::False),
                    other => Err(de::Error::unknown_variant(
                        other,
                        &[$true_wire, $false_wire],
                    )),
                }
            }
        }
    };
}

/// A documented native JSON boolean. The wrapper keeps it distinct from string flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WireBool(pub bool);

string_bool!(YnFlag, "Y", "N", "A provider `Y`/`N` string flag.");
string_bool!(YesNoFlag, "Yes", "No", "A provider `Yes`/`No` string flag.");
string_bool!(
    TrueFalseFlag,
    "true",
    "false",
    "A provider lowercase `true`/`false` string flag."
);
string_bool!(
    TitleCaseBoolFlag,
    "True",
    "False",
    "A provider title-case `True`/`False` string flag."
);

/// Dynamic endpoint JSON, recursively represented without assuming object keys.
pub type DynamicJson = serde_json::Value;

/// Object-root dynamic endpoint JSON with arbitrary member names and value shapes.
///
/// This preserves JSON values, including arbitrary-precision integer tokens,
/// without treating object-member order as semantic. Duplicate member names
/// therefore follow the JSON map model and are not preserved independently.
/// No order-preservation dependency is enabled for this contract.
pub type DynamicObject = serde_json::Map<String, DynamicJson>;

/// A strict RFC 3339 timestamp that preserves its original offset and precision.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IsoTimestamp(String);

impl IsoTimestamp {
    pub fn parse(value: &str) -> Result<Self, InvalidTemporalValue> {
        DateTime::parse_from_rfc3339(value)
            .map(|_| Self(value.to_owned()))
            .map_err(|_| InvalidTemporalValue::new("RFC 3339 timestamp"))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IsoTimestamp {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for IsoTimestamp {
    type Err = InvalidTemporalValue;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for IsoTimestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for IsoTimestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(&String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// A field documented as either `YYYY-MM-DD` or `YYYY-MM-DD HH:MM:SS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DateOrDateTime {
    Date(Date),
    DateTime(ApiDateTime),
}

impl DateOrDateTime {
    pub const fn as_date(self) -> Option<Date> {
        match self {
            Self::Date(value) => Some(value),
            Self::DateTime(_) => None,
        }
    }

    pub const fn as_datetime(self) -> Option<ApiDateTime> {
        match self {
            Self::Date(_) => None,
            Self::DateTime(value) => Some(value),
        }
    }
}

impl fmt::Display for DateOrDateTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Date(value) => value.fmt(formatter),
            Self::DateTime(value) => value.fmt(formatter),
        }
    }
}

impl FromStr for DateOrDateTime {
    type Err = InvalidTemporalValue;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() == 10 {
            Date::parse(value).map(Self::Date)
        } else if value.len() == 19 {
            ApiDateTime::parse(value).map(Self::DateTime)
        } else {
            Err(InvalidTemporalValue::new(
                "YYYY-MM-DD date or YYYY-MM-DD HH:MM:SS datetime",
            ))
        }
    }
}

impl Serialize for DateOrDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for DateOrDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer)?
            .parse()
            .map_err(de::Error::custom)
    }
}

/// A US-formatted date represented exactly as `MM-DD-YYYY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UsDate(NaiveDate);

impl UsDate {
    pub fn parse(value: &str) -> Result<Self, InvalidTemporalValue> {
        const EXPECTED: &str = "MM-DD-YYYY date";
        let exact_shape = value.len() == 10
            && value.is_ascii()
            && value.bytes().enumerate().all(|(index, byte)| match index {
                2 | 5 => byte == b'-',
                _ => byte.is_ascii_digit(),
            });
        if !exact_shape {
            return Err(InvalidTemporalValue::new(EXPECTED));
        }
        NaiveDate::parse_from_str(value, "%m-%d-%Y")
            .map(Self)
            .map_err(|_| InvalidTemporalValue::new(EXPECTED))
    }

    /// Returns the inner calendar date.
    pub const fn into_inner(self) -> NaiveDate {
        self.0
    }
}

impl fmt::Display for UsDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.format("%m-%d-%Y"))
    }
}

impl FromStr for UsDate {
    type Err = InvalidTemporalValue;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for UsDate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for UsDate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(&String::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// Opaque human-readable or partial date text, preserved without date parsing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OpaqueDateText(pub String);

macro_rules! optional_temporal_module {
    ($module:ident, $type:ty) => {
        pub mod $module {
            use super::*;

            pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<$type>, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = Option::<String>::deserialize(deserializer)?;
                value
                    .filter(|value| !value.is_empty())
                    .map(|value| value.parse().map_err(de::Error::custom))
                    .transpose()
            }

            pub fn serialize<S>(value: &Option<$type>, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                match value {
                    Some(value) => serializer.collect_str(value),
                    None => serializer.serialize_none(),
                }
            }
        }
    };
}

optional_temporal_module!(empty_or_null_date, Date);
optional_temporal_module!(empty_or_null_date_or_datetime, DateOrDateTime);

/// Codec for a required wire key whose absent date is represented by `""`.
///
/// Unlike [`empty_or_null_date`], JSON null is rejected and `None` serializes
/// back to the provider's exact empty-string sentinel. Do not add
/// `#[serde(default)]` at call sites when the response key is required.
pub mod empty_date {
    use super::*;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Date>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        if value.is_empty() {
            Ok(None)
        } else {
            value.parse().map(Some).map_err(de::Error::custom)
        }
    }

    pub fn serialize<S>(value: &Option<Date>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(value) => serializer.collect_str(value),
            None => serializer.serialize_str(""),
        }
    }
}

/// Serializer for a [`Volume`](crate::types::Volume) response field.
///
/// The provider sends volume as a JSON integer and, intermittently, as a
/// fractional number, so the field is an `f64` that deserializes from any
/// JSON number. Re-encoding writes a finite integral value back as a JSON
/// integer (so a documented row round-trips byte-for-byte) and anything else
/// as a JSON float. Apply it with `serialize_with`; deserialization is serde's
/// default `f64` path.
pub mod volume {
    use super::*;

    pub fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if value.is_finite() && value.fract() == 0.0 {
            if *value >= 0.0 && *value < u64::MAX as f64 {
                return serializer.serialize_u64(*value as u64);
            }
            if *value < 0.0 && *value > i64::MIN as f64 {
                return serializer.serialize_i64(*value as i64);
            }
        }
        serializer.serialize_f64(*value)
    }
}
