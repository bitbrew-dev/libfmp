//! Integer, decimal, and flag arguments.
//!
//! Integers are taken as `i64` and range-checked here so a negative or
//! oversized Python `int` raises `FmpValidationError` naming the keyword
//! instead of pyo3's `OverflowError`.

use libfmp::{
    codecs::TrueFalseFlag,
    query::{PeriodLength, Quarter, Year},
    types::{
        CalendarQuarter, CalendarYear, FiniteDecimal, InvalidCalendarQuarter, Limit,
        MarketCapitalization, Page, Volume,
    },
};
use pyo3::prelude::*;

use crate::errors::validation_error;

const U32_RANGE: &str = "must be an integer from 0 through 4294967295";
const NON_NEGATIVE: &str = "must be a non-negative integer";
const QUARTER_RANGE: &str = "quarter must be an integer from 1 through 4";
const PERIOD_LENGTH: &str = "period length must be a positive integer";

fn unsigned_u32(name: &str, value: i64) -> PyResult<u32> {
    u32::try_from(value).map_err(|_| validation_error(name, U32_RANGE))
}

fn unsigned_u64(name: &str, value: i64) -> PyResult<u64> {
    u64::try_from(value).map_err(|_| validation_error(name, NON_NEGATIVE))
}

macro_rules! u32_arg {
    ($($(#[doc = $doc:literal])* $function:ident => $target:ident),+ $(,)?) => {
        $(
            $(#[doc = $doc])*
            #[doc = ""]
            #[doc = "Accepts any Python `int` from 0 through 4294967295."]
            pub fn $function(name: &str, value: i64) -> PyResult<$target> {
                unsigned_u32(name, value).map($target)
            }
        )+
    };
}

u32_arg! {
    /// Converts a Python `int` into a result [`Limit`].
    limit => Limit,
    /// Converts a Python `int` into a page index [`Page`].
    page => Page,
    /// Converts a Python `int` into a query [`Year`].
    year => Year,
    /// Converts a Python `int` into a [`CalendarYear`].
    calendar_year => CalendarYear,
}

/// Converts a Python `int` from 1 through 4 into a query [`Quarter`].
pub fn quarter(name: &str, value: i64) -> PyResult<Quarter> {
    match value {
        1 => Ok(Quarter::Q1),
        2 => Ok(Quarter::Q2),
        3 => Ok(Quarter::Q3),
        4 => Ok(Quarter::Q4),
        _ => Err(validation_error(name, QUARTER_RANGE)),
    }
}

/// Converts a Python `int` from 1 through 4 into a [`CalendarQuarter`].
pub fn calendar_quarter(name: &str, value: i64) -> PyResult<CalendarQuarter> {
    u8::try_from(value)
        .ok()
        .and_then(|value| CalendarQuarter::new(value).ok())
        .ok_or_else(|| validation_error(name, InvalidCalendarQuarter))
}

/// Converts a positive Python `int` into a technical-indicator [`PeriodLength`].
pub fn period_length(name: &str, value: i64) -> PyResult<PeriodLength> {
    u32::try_from(value)
        .ok()
        .and_then(PeriodLength::new)
        .ok_or_else(|| validation_error(name, PERIOD_LENGTH))
}

/// Converts a non-negative Python `int` into a [`MarketCapitalization`].
pub fn market_capitalization(name: &str, value: i64) -> PyResult<MarketCapitalization> {
    unsigned_u64(name, value)
}

/// Converts a non-negative Python `int` into a [`Volume`].
pub fn volume(name: &str, value: i64) -> PyResult<Volume> {
    unsigned_u64(name, value)
}

/// Converts a Python `float` (or `int`) into a [`FiniteDecimal`].
///
/// `nan`, `inf`, and `-inf` are rejected.
pub fn finite_decimal(name: &str, value: f64) -> PyResult<FiniteDecimal> {
    FiniteDecimal::new(value).map_err(|error| validation_error(name, error))
}

/// Converts a Python `bool` into a lowercase `true`/`false` [`TrueFalseFlag`].
///
/// Infallible; the `PyResult` keeps the signature uniform with the other
/// conversions so it composes with [`optional`](crate::args::optional).
pub fn true_false_flag(_name: &str, value: bool) -> PyResult<TrueFalseFlag> {
    Ok(if value {
        TrueFalseFlag::True
    } else {
        TrueFalseFlag::False
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::testing::validation_message;

    #[test]
    fn u32_arguments_accept_range_and_reject_negative() {
        crate::args::testing::init();
        assert_eq!(limit("limit", 0).expect("zero"), Limit(0));
        assert_eq!(page("page", 7).expect("valid"), Page(7));
        assert_eq!(year("year", 2024).expect("valid"), Year(2024));
        assert_eq!(
            calendar_year("year", 2024).expect("valid"),
            CalendarYear(2024)
        );
        assert_eq!(
            limit("limit", i64::from(u32::MAX)).expect("max"),
            Limit(u32::MAX)
        );
        let error = limit("limit", -1).expect_err("negative");
        assert_eq!(
            validation_message(error),
            "limit: must be an integer from 0 through 4294967295"
        );
        let error = page("page", i64::from(u32::MAX) + 1).expect_err("overflow");
        assert_eq!(
            validation_message(error),
            "page: must be an integer from 0 through 4294967295"
        );
    }

    #[test]
    fn quarter_maps_one_through_four() {
        crate::args::testing::init();
        assert_eq!(quarter("quarter", 1).expect("q1"), Quarter::Q1);
        assert_eq!(quarter("quarter", 4).expect("q4"), Quarter::Q4);
        assert_eq!(quarter("quarter", 4).expect("q4").as_str(), "4");
        let error = quarter("quarter", 5).expect_err("out of range");
        assert_eq!(
            validation_message(error),
            "quarter: quarter must be an integer from 1 through 4"
        );
        let error = quarter("quarter", 0).expect_err("zero");
        assert_eq!(
            validation_message(error),
            "quarter: quarter must be an integer from 1 through 4"
        );
    }

    #[test]
    fn calendar_quarter_validates_range() {
        crate::args::testing::init();
        assert_eq!(calendar_quarter("quarter", 2).expect("valid").get(), 2);
        for invalid in [0, 5, -1, 300] {
            let error = calendar_quarter("quarter", invalid).expect_err("out of range");
            assert_eq!(
                validation_message(error),
                "quarter: calendar quarter must be an integer from 1 through 4"
            );
        }
    }

    #[test]
    fn period_length_requires_positive() {
        crate::args::testing::init();
        assert_eq!(period_length("period", 14).expect("valid").get(), 14);
        for invalid in [0, -3, i64::from(u32::MAX) + 1] {
            let error = period_length("period", invalid).expect_err("invalid");
            assert_eq!(
                validation_message(error),
                "period: period length must be a positive integer"
            );
        }
    }

    #[test]
    fn u64_filters_reject_negative() {
        crate::args::testing::init();
        assert_eq!(
            market_capitalization("market_cap_more_than", 1_000_000).expect("valid"),
            1_000_000
        );
        assert_eq!(volume("volume_more_than", 0).expect("zero"), 0);
        let error = volume("volume_more_than", -5).expect_err("negative");
        assert_eq!(
            validation_message(error),
            "volume_more_than: must be a non-negative integer"
        );
    }

    #[test]
    fn finite_decimal_rejects_non_finite() {
        crate::args::testing::init();
        assert_eq!(finite_decimal("beta", 1.5).expect("valid").get(), 1.5);
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let error = finite_decimal("beta", invalid).expect_err("non-finite");
            assert_eq!(
                validation_message(error),
                "beta: decimal value must be finite"
            );
        }
    }

    #[test]
    fn true_false_flag_maps_bool() {
        crate::args::testing::init();
        assert_eq!(
            true_false_flag("invalid", true).expect("infallible"),
            TrueFalseFlag::True
        );
        assert_eq!(
            true_false_flag("invalid", false)
                .expect("infallible")
                .as_str(),
            "false"
        );
    }
}
