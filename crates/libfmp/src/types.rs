//! Fundamental identifiers and wire-level value types.

use std::{error::Error, fmt, str::FromStr};

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

/// Why a string-backed identifier or code was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StringValueError {
    /// The value was empty or contained only whitespace.
    Empty,
    /// The value contained a control character.
    ControlCharacter,
    /// A ticker contained a comma, which would make list encoding ambiguous.
    Comma,
}

impl fmt::Display for StringValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("value must not be empty or whitespace-only"),
            Self::ControlCharacter => {
                formatter.write_str("value must not contain control characters")
            }
            Self::Comma => formatter.write_str("ticker must not contain a comma"),
        }
    }
}

impl Error for StringValueError {}

fn validate_string_value(value: &str, reject_comma: bool) -> Result<(), StringValueError> {
    if value.trim().is_empty() {
        return Err(StringValueError::Empty);
    }
    if value.chars().any(char::is_control) {
        return Err(StringValueError::ControlCharacter);
    }
    if reject_comma && value.contains(',') {
        return Err(StringValueError::Comma);
    }
    Ok(())
}

macro_rules! string_value {
    ($name:ident, $description:literal, $reject_comma:literal) => {
        #[doc = $description]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            /// Validates and constructs a value while preserving its representation.
            pub fn new(value: impl Into<String>) -> Result<Self, StringValueError> {
                let value = value.into();
                validate_string_value(&value, $reject_comma)?;
                Ok(Self(value))
            }

            /// Borrows the original, unnormalized representation.
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Returns the original, unnormalized representation.
            pub fn into_inner(self) -> String {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = StringValueError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }

        impl TryFrom<String> for $name {
            type Error = StringValueError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = StringValueError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(de::Error::custom)
            }
        }
    };
}

string_value!(Ticker, "A provider ticker symbol.", true);
string_value!(
    TipRanksExpertUid,
    "A representation-preserving stable TipRanks analyst identifier.",
    false
);
string_value!(
    BulkPart,
    "An open, representation-preserving provider bulk partition.",
    false
);
string_value!(
    SearchTerm,
    "A representation-preserving provider search term.",
    false
);
string_value!(Cik, "A Central Index Key.", false);
string_value!(
    CongressionalMemberId,
    "A representation-preserving congressional member identifier.",
    false
);
string_value!(
    FormType,
    "An open, representation-preserving SEC filing form type.",
    false
);
string_value!(
    TransactionTypeCode,
    "An open, representation-preserving insider transaction type code.",
    false
);
string_value!(Cusip, "A CUSIP identifier.", false);
string_value!(Isin, "An ISIN identifier.", false);
string_value!(Lei, "A Legal Entity Identifier.", false);
string_value!(ExchangeCode, "An open provider exchange code.", false);
string_value!(
    MarketHoursTimestamp,
    "An opaque, representation-preserving market-hours query timestamp.",
    false
);
string_value!(CurrencyCode, "An open provider currency code.", false);
string_value!(CountryCode, "An open provider country code.", false);
string_value!(Sector, "An open provider company sector.", false);
string_value!(Industry, "An open provider company industry.", false);
string_value!(
    BenchmarkYear,
    "An open, representation-preserving executive-compensation benchmark query year.",
    false
);

/// Error returned when a query decimal is NaN or infinite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NonFiniteDecimal;

impl fmt::Display for NonFiniteDecimal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("decimal value must be finite")
    }
}

impl Error for NonFiniteDecimal {}

/// A finite decimal suitable for inclusion in a request URL.
///
/// This deliberately imposes no sign or range restriction: endpoint
/// documentation, rather than the shared scalar, owns those constraints.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct FiniteDecimal(f64);

impl FiniteDecimal {
    /// Validates and constructs a finite decimal.
    pub fn new(value: f64) -> Result<Self, NonFiniteDecimal> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(NonFiniteDecimal)
        }
    }

    /// Returns the validated primitive value.
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for FiniteDecimal {
    type Error = NonFiniteDecimal;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<FiniteDecimal> for f64 {
    fn from(value: FiniteDecimal) -> Self {
        value.get()
    }
}

impl fmt::Display for FiniteDecimal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Serialize for FiniteDecimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_f64(self.0)
    }
}

impl<'de> Deserialize<'de> for FiniteDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(f64::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// Error returned when constructing an empty ticker list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmptyTickerList;

impl fmt::Display for EmptyTickerList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ticker list must contain at least one ticker")
    }
}

impl Error for EmptyTickerList {}

/// A non-empty sequence of tickers encoded as a comma-separated query value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TickerList(Vec<Ticker>);

impl TickerList {
    pub fn new(tickers: Vec<Ticker>) -> Result<Self, EmptyTickerList> {
        if tickers.is_empty() {
            Err(EmptyTickerList)
        } else {
            Ok(Self(tickers))
        }
    }

    pub fn as_slice(&self) -> &[Ticker] {
        &self.0
    }

    pub fn into_inner(self) -> Vec<Ticker> {
        self.0
    }

    pub fn iter(&self) -> impl ExactSizeIterator<Item = &Ticker> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl TryFrom<Vec<Ticker>> for TickerList {
    type Error = EmptyTickerList;

    fn try_from(tickers: Vec<Ticker>) -> Result<Self, Self::Error> {
        Self::new(tickers)
    }
}

impl fmt::Display for TickerList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tickers = self.0.iter();
        if let Some(first) = tickers.next() {
            first.fmt(formatter)?;
        }
        for ticker in tickers {
            formatter.write_str(",")?;
            ticker.fmt(formatter)?;
        }
        Ok(())
    }
}

/// Error returned when a date or API datetime does not match its wire format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTemporalValue {
    expected: &'static str,
}

impl InvalidTemporalValue {
    pub(crate) const fn new(expected: &'static str) -> Self {
        Self { expected }
    }

    pub fn expected(&self) -> &'static str {
        self.expected
    }
}

impl fmt::Display for InvalidTemporalValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "value must be a valid {}", self.expected)
    }
}

impl Error for InvalidTemporalValue {}

fn has_exact_ascii_shape(value: &str, len: usize, separators: &[(usize, u8)]) -> bool {
    if value.len() != len || !value.is_ascii() {
        return false;
    }

    value.bytes().enumerate().all(|(index, byte)| {
        separators
            .iter()
            .find_map(|(position, expected)| (*position == index).then_some(*expected))
            .map_or_else(|| byte.is_ascii_digit(), |expected| byte == expected)
    })
}

/// A date represented on the wire as exactly `YYYY-MM-DD`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date(NaiveDate);

impl Date {
    pub fn parse(value: &str) -> Result<Self, InvalidTemporalValue> {
        const EXPECTED: &str = "YYYY-MM-DD date";
        if !has_exact_ascii_shape(value, 10, &[(4, b'-'), (7, b'-')]) {
            return Err(InvalidTemporalValue { expected: EXPECTED });
        }
        NaiveDate::parse_from_str(value, "%Y-%m-%d")
            .map(Self)
            .map_err(|_| InvalidTemporalValue { expected: EXPECTED })
    }

    /// Returns the inner calendar date.
    pub const fn into_inner(self) -> NaiveDate {
        self.0
    }
}

impl FromStr for Date {
    type Err = InvalidTemporalValue;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.format("%Y-%m-%d"))
    }
}

impl Serialize for Date {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Date {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(de::Error::custom)
    }
}

/// A timezone-less API datetime represented as exactly `YYYY-MM-DD HH:MM:SS`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ApiDateTime(NaiveDateTime);

impl ApiDateTime {
    pub fn parse(value: &str) -> Result<Self, InvalidTemporalValue> {
        const EXPECTED: &str = "YYYY-MM-DD HH:MM:SS datetime";
        let separators = &[(4, b'-'), (7, b'-'), (10, b' '), (13, b':'), (16, b':')];
        if !has_exact_ascii_shape(value, 19, separators) {
            return Err(InvalidTemporalValue { expected: EXPECTED });
        }
        NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S")
            .map(Self)
            .map_err(|_| InvalidTemporalValue { expected: EXPECTED })
    }

    /// Returns the inner naive datetime.
    pub const fn into_inner(self) -> NaiveDateTime {
        self.0
    }
}

impl FromStr for ApiDateTime {
    type Err = InvalidTemporalValue;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl fmt::Display for ApiDateTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.format("%Y-%m-%d %H:%M:%S"))
    }
}

impl Serialize for ApiDateTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for ApiDateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(de::Error::custom)
    }
}

/// A Unix timestamp measured in seconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnixSeconds(pub i64);

/// A Unix timestamp measured in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UnixMilliseconds(pub i64);

impl fmt::Display for UnixSeconds {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for UnixMilliseconds {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A numeric calendar year received in a response.
///
/// This is distinct from string-backed fiscal years and query-year units so
/// the provider's JSON number representation remains part of the contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CalendarYear(pub u32);

impl CalendarYear {
    /// Returns the primitive numeric year.
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl From<u32> for CalendarYear {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<CalendarYear> for u32 {
    fn from(value: CalendarYear) -> Self {
        value.get()
    }
}

impl fmt::Display for CalendarYear {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// A numeric calendar quarter received in a response.
///
/// This is distinct from [`crate::query::Quarter`], whose textual `1` through
/// `4` representations are query values. Response quarters are JSON integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CalendarQuarter(u8);

impl CalendarQuarter {
    /// Validates and constructs a numeric response quarter.
    pub const fn new(value: u8) -> Result<Self, InvalidCalendarQuarter> {
        if value >= 1 && value <= 4 {
            Ok(Self(value))
        } else {
            Err(InvalidCalendarQuarter)
        }
    }

    /// Returns the primitive numeric quarter.
    pub const fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for CalendarQuarter {
    type Error = InvalidCalendarQuarter;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<CalendarQuarter> for u8 {
    fn from(value: CalendarQuarter) -> Self {
        value.get()
    }
}

impl fmt::Display for CalendarQuarter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Serialize for CalendarQuarter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.get())
    }
}

impl<'de> Deserialize<'de> for CalendarQuarter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(u8::deserialize(deserializer)?).map_err(de::Error::custom)
    }
}

/// Why a numeric response quarter was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidCalendarQuarter;

impl fmt::Display for InvalidCalendarQuarter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("calendar quarter must be an integer from 1 through 4")
    }
}

impl Error for InvalidCalendarQuarter {}

/// A representation-preserving provider page index.
///
/// Construction does not enforce endpoint-specific bounds. Documented bounds
/// are exposed as advisory endpoint metadata with opt-in `accepts_page` checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Page(pub u32);

/// A representation-preserving provider result limit.
///
/// Construction does not enforce endpoint-specific bounds. Documented bounds
/// are exposed as advisory endpoint metadata with opt-in `accepts_limit` checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Limit(pub u32);

impl fmt::Display for Page {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl fmt::Display for Limit {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Error returned when an inclusive date range starts after it ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidDateRange;

impl fmt::Display for InvalidDateRange {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("date range start must not be later than its end")
    }
}

impl Error for InvalidDateRange {}

/// An inclusive date range whose start is not later than its end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DateRange {
    from: Date,
    to: Date,
}

impl DateRange {
    pub fn new(from: Date, to: Date) -> Result<Self, InvalidDateRange> {
        if from > to {
            Err(InvalidDateRange)
        } else {
            Ok(Self { from, to })
        }
    }

    pub fn from(&self) -> Date {
        self.from
    }

    pub fn to(&self) -> Date {
        self.to
    }

    /// Returns the elapsed calendar days between the inclusive endpoints.
    ///
    /// Equal endpoints have a span of zero days. This supports optional checks
    /// against the advisory maximum date range in endpoint metadata.
    pub fn span_days(&self) -> u64 {
        self.to.0.signed_duration_since(self.from.0).num_days() as u64
    }
}

/// A price represented by the provider as a JSON number.
pub type Price = f64;
/// A signed change represented by the provider as a JSON number.
pub type Change = f64;
/// A percentage represented by the provider as a JSON number.
pub type Percentage = f64;
/// A market value represented by the provider as a JSON number.
pub type MarketValue = f64;
/// A traded volume represented by the provider as a JSON number.
///
/// The provider documents volume as a non-negative integer, but it sends a
/// JSON integer or, intermittently, a fractional number (`20201922.82733` was
/// observed on `quote-short`), so the type is `f64` and the SDK accepts any
/// JSON number without enforcing a sign or integrality. Response fields
/// re-encode an integral value as a JSON integer through
/// the crate's `volume` serde codec; the screener volume filters stay `u64`.
pub type Volume = f64;
/// A non-negative count represented by the provider as a JSON integer.
pub type Count = u64;
/// A market capitalization represented by the provider as a JSON integer.
pub type MarketCapitalization = u64;
/// A signed currency amount reported in a financial statement.
///
/// Statement values can be negative (for example, expenses and cash outflows),
/// so this deliberately differs from non-negative market-cap and volume types.
pub type StatementAmount = i64;
