//! Date and datetime arguments: `datetime.date | str`, `datetime.datetime | str`,
//! and `(date, date)` ranges.

use chrono::{NaiveDate, NaiveDateTime};
use libfmp::types::{ApiDateTime, Date, DateRange};
use pyo3::prelude::*;
use pyo3_stub_gen::impl_stub_type;

use crate::errors::validation_error;

/// A `datetime.date | str` argument.
///
/// A `str` must be exactly `YYYY-MM-DD`. A `datetime.datetime` is also
/// accepted (it subclasses `date`) and its time component is dropped.
#[derive(Debug, Clone, PartialEq, Eq, FromPyObject)]
pub enum DateArg {
    #[pyo3(transparent, annotation = "datetime.date")]
    Date(NaiveDate),
    #[pyo3(transparent, annotation = "str")]
    Text(String),
}

impl_stub_type!(DateArg = NaiveDate | String);

/// A `datetime.datetime | str` argument.
///
/// A `str` must be exactly `YYYY-MM-DD HH:MM:SS`. A `datetime.datetime` must
/// be naive (no `tzinfo`; pyo3 raises `TypeError` otherwise) and any
/// microseconds are truncated because the wire format carries whole seconds.
#[derive(Debug, Clone, PartialEq, Eq, FromPyObject)]
pub enum DateTimeArg {
    #[pyo3(transparent, annotation = "datetime.datetime")]
    DateTime(NaiveDateTime),
    #[pyo3(transparent, annotation = "str")]
    Text(String),
}

impl_stub_type!(DateTimeArg = NaiveDateTime | String);

/// Converts a `datetime.date | str` argument into a wire [`Date`].
pub fn date(name: &str, value: DateArg) -> PyResult<Date> {
    let text = match value {
        DateArg::Date(date) => date.format("%Y-%m-%d").to_string(),
        DateArg::Text(text) => text,
    };
    Date::parse(&text).map_err(|error| validation_error(name, error))
}

/// Converts a `datetime.datetime | str` argument into a wire [`ApiDateTime`].
pub fn api_datetime(name: &str, value: DateTimeArg) -> PyResult<ApiDateTime> {
    let text = match value {
        DateTimeArg::DateTime(datetime) => datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
        DateTimeArg::Text(text) => text,
    };
    ApiDateTime::parse(&text).map_err(|error| validation_error(name, error))
}

/// Converts a `(date, date)` pair into an inclusive [`DateRange`].
///
/// Element errors name the position, for example `period[1]: ...`; an
/// inverted range names the whole argument.
pub fn date_range(name: &str, value: (DateArg, DateArg)) -> PyResult<DateRange> {
    let from = date(&format!("{name}[0]"), value.0)?;
    let to = date(&format!("{name}[1]"), value.1)?;
    DateRange::new(from, to).map_err(|error| validation_error(name, error))
}

#[cfg(test)]
mod tests {
    use pyo3::types::PyString;

    use super::*;
    use crate::args::testing::{validation_message, with_py};

    fn py_date<'py>(py: Python<'py>, kind: &str, parts: &[i64]) -> Bound<'py, PyAny> {
        let parts = pyo3::types::PyTuple::new(py, parts).expect("tuple");
        py.import("datetime")
            .expect("datetime module")
            .getattr(kind)
            .expect("class")
            .call1(parts)
            .expect("constructed")
    }

    #[test]
    fn date_accepts_iso_string() {
        crate::args::testing::init();
        let converted = date("from", DateArg::Text("2024-01-31".into())).expect("valid");
        assert_eq!(converted.to_string(), "2024-01-31");
    }

    #[test]
    fn date_rejects_malformed_string() {
        crate::args::testing::init();
        for invalid in ["2024-1-31", "20240131", "2024-02-30", "yesterday"] {
            let error = date("from", DateArg::Text(invalid.into())).expect_err("invalid");
            assert_eq!(
                validation_message(error),
                "from: value must be a valid YYYY-MM-DD date"
            );
        }
    }

    #[test]
    fn date_accepts_python_date_and_datetime() {
        crate::args::testing::init();
        with_py(|py| {
            let value: DateArg = py_date(py, "date", &[2024, 1, 31]).extract().expect("date");
            assert_eq!(
                date("from", value).expect("valid").to_string(),
                "2024-01-31"
            );
            let value: DateArg = py_date(py, "datetime", &[2024, 1, 31, 9, 30, 0])
                .extract()
                .expect("datetime is a date");
            assert_eq!(
                date("from", value).expect("valid").to_string(),
                "2024-01-31"
            );
            let value: DateArg = PyString::new(py, "2024-01-31").extract().expect("str");
            assert_eq!(value, DateArg::Text("2024-01-31".into()));
            let error = 5i64
                .into_pyobject(py)
                .expect("int")
                .extract::<DateArg>()
                .expect_err("int is neither");
            assert!(error.is_instance_of::<pyo3::exceptions::PyTypeError>(py));
        });
    }

    #[test]
    fn api_datetime_accepts_string_and_naive_datetime() {
        crate::args::testing::init();
        let converted =
            api_datetime("since", DateTimeArg::Text("2024-01-31 09:30:00".into())).expect("valid");
        assert_eq!(converted.to_string(), "2024-01-31 09:30:00");
        with_py(|py| {
            let value: DateTimeArg = py_date(py, "datetime", &[2024, 1, 31, 9, 30, 5, 999])
                .extract()
                .expect("naive datetime");
            assert_eq!(
                api_datetime("since", value).expect("valid").to_string(),
                "2024-01-31 09:30:05"
            );
            let aware = py_date(py, "datetime", &[2024, 1, 31, 9, 30, 0])
                .call_method1("replace", ())
                .expect("replace");
            let utc = py
                .import("datetime")
                .expect("datetime")
                .getattr("timezone")
                .expect("timezone")
                .getattr("utc")
                .expect("utc");
            let kwargs = pyo3::types::PyDict::new(py);
            kwargs.set_item("tzinfo", utc).expect("kwarg");
            let aware = aware
                .call_method("replace", (), Some(&kwargs))
                .expect("aware datetime");
            let error = aware
                .extract::<DateTimeArg>()
                .expect_err("tz-aware rejected");
            assert!(error.is_instance_of::<pyo3::exceptions::PyTypeError>(py));
        });
    }

    #[test]
    fn api_datetime_rejects_malformed_string() {
        crate::args::testing::init();
        for invalid in ["2024-01-31", "2024-01-31T09:30:00", "2024-01-31 09:30"] {
            let error =
                api_datetime("since", DateTimeArg::Text(invalid.into())).expect_err("invalid");
            assert_eq!(
                validation_message(error),
                "since: value must be a valid YYYY-MM-DD HH:MM:SS datetime"
            );
        }
    }

    #[test]
    fn date_range_orders_endpoints() {
        crate::args::testing::init();
        let range = date_range(
            "period",
            (
                DateArg::Text("2024-01-01".into()),
                DateArg::Text("2024-01-31".into()),
            ),
        )
        .expect("valid");
        assert_eq!(range.span_days(), 30);
        let error = date_range(
            "period",
            (
                DateArg::Text("2024-02-01".into()),
                DateArg::Text("2024-01-31".into()),
            ),
        )
        .expect_err("inverted");
        assert_eq!(
            validation_message(error),
            "period: date range start must not be later than its end"
        );
        let error = date_range(
            "period",
            (
                DateArg::Text("2024-01-01".into()),
                DateArg::Text("nope".into()),
            ),
        )
        .expect_err("bad end");
        assert_eq!(
            validation_message(error),
            "period[1]: value must be a valid YYYY-MM-DD date"
        );
    }
}
