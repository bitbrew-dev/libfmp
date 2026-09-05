//! Closed wire vocabularies and open indicator names: `str -> <enum>`.
//!
//! Closed enums accept their exact provider spelling case-insensitively, plus
//! the aliases listed on each function. Rejections list every accepted wire
//! spelling so the Python caller can correct the value without consulting
//! the provider documentation.

use libfmp::{
    endpoints::statements::SegmentationStructure,
    query::{
        ChartTimeframe, EconomicIndicator, FiscalPeriod, OpenEconomicIndicator, RetrievalFrequency,
        StatementPeriod,
    },
};
use pyo3::prelude::*;

use crate::errors::validation_error;

const FISCAL_PERIODS: [(&str, FiscalPeriod); 5] = [
    ("Q1", FiscalPeriod::Q1),
    ("Q2", FiscalPeriod::Q2),
    ("Q3", FiscalPeriod::Q3),
    ("Q4", FiscalPeriod::Q4),
    ("FY", FiscalPeriod::FullYear),
];

const RETRIEVAL_FREQUENCIES: [(&str, RetrievalFrequency); 3] = [
    ("annual", RetrievalFrequency::Annual),
    ("quarter", RetrievalFrequency::Quarterly),
    ("quarterly", RetrievalFrequency::Quarterly),
];

const CHART_TIMEFRAMES: [(&str, ChartTimeframe); 7] = [
    ("1min", ChartTimeframe::OneMinute),
    ("5min", ChartTimeframe::FiveMinutes),
    ("15min", ChartTimeframe::FifteenMinutes),
    ("30min", ChartTimeframe::ThirtyMinutes),
    ("1hour", ChartTimeframe::OneHour),
    ("4hour", ChartTimeframe::FourHours),
    ("1day", ChartTimeframe::OneDay),
];

const SEGMENTATION_STRUCTURES: [(&str, SegmentationStructure); 1] =
    [("flat", SegmentationStructure::Flat)];

fn lookup<T: Copy>(table: &[(&str, T)], value: &str) -> Option<T> {
    table
        .iter()
        .find(|(wire, _)| wire.eq_ignore_ascii_case(value))
        .map(|(_, variant)| *variant)
}

fn closed_enum<T: Copy>(
    name: &str,
    value: &str,
    label: &str,
    tables: &[&[(&str, T)]],
    accepted: &[&str],
) -> PyResult<T> {
    tables
        .iter()
        .find_map(|table| lookup(table, value))
        .ok_or_else(|| {
            validation_error(
                name,
                format!("{label} must be one of {}", accepted.join(", ")),
            )
        })
}

/// Converts a Python `str` into a [`FiscalPeriod`].
///
/// Accepts `Q1`, `Q2`, `Q3`, `Q4`, and `FY`, case-insensitively.
pub fn fiscal_period(name: &str, value: &str) -> PyResult<FiscalPeriod> {
    closed_enum(
        name,
        value,
        "fiscal period",
        &[&FISCAL_PERIODS],
        &["Q1", "Q2", "Q3", "Q4", "FY"],
    )
}

/// Converts a Python `str` into a [`RetrievalFrequency`].
///
/// Accepts `annual` and `quarter` (alias `quarterly`), case-insensitively.
pub fn retrieval_frequency(name: &str, value: &str) -> PyResult<RetrievalFrequency> {
    closed_enum(
        name,
        value,
        "retrieval frequency",
        &[&RETRIEVAL_FREQUENCIES],
        &["annual", "quarter"],
    )
}

/// Converts a Python `str` into the seven-value [`StatementPeriod`] union.
///
/// Accepts every [`fiscal_period`] and [`retrieval_frequency`] spelling.
pub fn statement_period(name: &str, value: &str) -> PyResult<StatementPeriod> {
    if let Some(period) = lookup(&FISCAL_PERIODS, value) {
        return Ok(period.into());
    }
    if let Some(frequency) = lookup(&RETRIEVAL_FREQUENCIES, value) {
        return Ok(frequency.into());
    }
    Err(validation_error(
        name,
        "statement period must be one of Q1, Q2, Q3, Q4, FY, annual, quarter",
    ))
}

/// Converts a Python `str` into a technical-indicator [`ChartTimeframe`].
///
/// Accepts `1min`, `5min`, `15min`, `30min`, `1hour`, `4hour`, and `1day`,
/// case-insensitively.
pub fn chart_timeframe(name: &str, value: &str) -> PyResult<ChartTimeframe> {
    closed_enum(
        name,
        value,
        "timeframe",
        &[&CHART_TIMEFRAMES],
        &["1min", "5min", "15min", "30min", "1hour", "4hour", "1day"],
    )
}

/// Converts a Python `str` into a revenue [`SegmentationStructure`].
///
/// The provider documents only `flat`, matched case-insensitively.
pub fn segmentation_structure(name: &str, value: &str) -> PyResult<SegmentationStructure> {
    closed_enum(
        name,
        value,
        "segmentation structure",
        &[&SEGMENTATION_STRUCTURES],
        &["flat"],
    )
}

/// Converts a Python `str` into an [`EconomicIndicator`].
///
/// The 24 documented names are matched exactly (they are case-sensitive on
/// the wire, for example `GDP` and `realGDP`); any other non-empty value is
/// preserved as an open indicator for forward compatibility.
pub fn economic_indicator(name: &str, value: &str) -> PyResult<EconomicIndicator> {
    EconomicIndicator::new(value).map_err(|error| validation_error(name, error))
}

/// Converts a Python `str` into an [`OpenEconomicIndicator`].
pub fn open_economic_indicator(name: &str, value: &str) -> PyResult<OpenEconomicIndicator> {
    OpenEconomicIndicator::new(value).map_err(|error| validation_error(name, error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::testing::{init, validation_message};

    #[test]
    fn fiscal_period_is_case_insensitive() {
        init();
        assert_eq!(
            fiscal_period("period", "q1").expect("valid"),
            FiscalPeriod::Q1
        );
        assert_eq!(
            fiscal_period("period", "FY").expect("valid"),
            FiscalPeriod::FullYear
        );
        let error = fiscal_period("period", "annual").expect_err("not fiscal");
        assert_eq!(
            validation_message(error),
            "period: fiscal period must be one of Q1, Q2, Q3, Q4, FY"
        );
    }

    #[test]
    fn retrieval_frequency_accepts_quarterly_alias() {
        init();
        assert_eq!(
            retrieval_frequency("period", "Annual").expect("valid"),
            RetrievalFrequency::Annual
        );
        assert_eq!(
            retrieval_frequency("period", "quarterly").expect("alias"),
            RetrievalFrequency::Quarterly
        );
        assert_eq!(
            retrieval_frequency("period", "quarter")
                .expect("wire")
                .as_str(),
            "quarter"
        );
        let error = retrieval_frequency("period", "Q1").expect_err("not a frequency");
        assert_eq!(
            validation_message(error),
            "period: retrieval frequency must be one of annual, quarter"
        );
    }

    #[test]
    fn statement_period_covers_both_families() {
        init();
        assert_eq!(
            statement_period("period", "Q3").expect("fiscal"),
            StatementPeriod::Fiscal(FiscalPeriod::Q3)
        );
        assert_eq!(
            statement_period("period", "quarterly").expect("frequency alias"),
            StatementPeriod::Frequency(RetrievalFrequency::Quarterly)
        );
        assert_eq!(
            statement_period("period", "fy")
                .expect("lowercase")
                .to_string(),
            "FY"
        );
        let error = statement_period("period", "monthly").expect_err("unknown");
        assert_eq!(
            validation_message(error),
            "period: statement period must be one of Q1, Q2, Q3, Q4, FY, annual, quarter"
        );
    }

    #[test]
    fn chart_timeframe_matches_wire_spellings() {
        init();
        assert_eq!(
            chart_timeframe("timeframe", "1MIN").expect("valid"),
            ChartTimeframe::OneMinute
        );
        assert_eq!(
            chart_timeframe("timeframe", "4hour")
                .expect("valid")
                .as_str(),
            "4hour"
        );
        let error = chart_timeframe("timeframe", "2hour").expect_err("unknown");
        assert_eq!(
            validation_message(error),
            "timeframe: timeframe must be one of 1min, 5min, 15min, 30min, 1hour, 4hour, 1day"
        );
    }

    #[test]
    fn segmentation_structure_only_flat() {
        init();
        assert_eq!(
            segmentation_structure("structure", "Flat").expect("valid"),
            SegmentationStructure::Flat
        );
        let error = segmentation_structure("structure", "nested").expect_err("unknown");
        assert_eq!(
            validation_message(error),
            "structure: segmentation structure must be one of flat"
        );
    }

    #[test]
    fn economic_indicator_is_open_but_validated() {
        init();
        assert_eq!(
            economic_indicator("name", "GDP").expect("documented"),
            EconomicIndicator::Gdp
        );
        let open = economic_indicator("name", "newIndicator").expect("open");
        assert_eq!(open.as_str(), "newIndicator");
        assert!(matches!(open, EconomicIndicator::Other(_)));
        let error = economic_indicator("name", "").expect_err("empty");
        assert_eq!(
            validation_message(error),
            "name: value must not be empty or whitespace-only"
        );
        let error = open_economic_indicator("name", "bad\nname").expect_err("control");
        assert_eq!(
            validation_message(error),
            "name: value must not contain control characters"
        );
        assert_eq!(
            open_economic_indicator("name", "CPI")
                .expect("valid")
                .as_str(),
            "CPI"
        );
    }
}
