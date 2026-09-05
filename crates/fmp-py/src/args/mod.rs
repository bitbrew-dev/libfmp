//! Conversions from Python argument values into libfmp query primitives.
//!
//! Generated namespace methods never hand-roll a conversion: every parameter
//! a libfmp query constructor or `with_*` setter accepts has one function
//! here. Each function takes the Python keyword name first, so a rejected
//! value raises [`FmpValidationError`](crate::errors::FmpValidationError)
//! whose message reads `"{name}: {reason}"` and whose `category` attribute is
//! `"validation"`.
//!
//! The layer owns value validation only. A wrong Python type (an `int` where
//! a `str` is expected) is still reported by pyo3 as `TypeError` before these
//! functions run, matching the convention that `TypeError` is for shapes and
//! `ValueError`-like exceptions are for contents.
//!
//! Emitter usage, one line per parameter:
//!
//! ```ignore
//! let symbol = args::ticker("symbol", symbol)?;
//! let limit = args::optional("limit", limit, args::limit)?;
//! ```
//!
//! Union-shaped inputs (`datetime.date | str`, `str | list[str]`) are typed
//! enums with a derived `FromPyObject` and a `PyStubType`, so they can appear
//! directly in `#[gen_stub_pymethods]` signatures.

mod text;

use pyo3::prelude::*;

pub use text::{
    SymbolsArg, benchmark_year, bulk_part, cik, congressional_member_id, country_code,
    currency_code, cusip, exchange_code, form_type, industry, isin, lei, market_hours_timestamp,
    search_term, sector, ticker, ticker_list, tipranks_expert_uid, transaction_type_code,
};

/// Applies a conversion to an optional keyword argument.
///
/// `None` passes through unchanged; `Some(value)` is converted with `convert`
/// and any validation error names `name`.
pub fn optional<I, T>(
    name: &str,
    value: Option<I>,
    convert: impl FnOnce(&str, I) -> PyResult<T>,
) -> PyResult<Option<T>> {
    value.map(|value| convert(name, value)).transpose()
}

#[cfg(test)]
pub(crate) mod testing {
    use pyo3::prelude::*;

    use crate::errors::FmpValidationError;

    /// Asserts that `error` is an `FmpValidationError` carrying the
    /// validation category, and returns its message for text assertions.
    pub(crate) fn validation_message(error: PyErr) -> String {
        Python::initialize();
        Python::attach(|py| {
            assert!(
                error.is_instance_of::<FmpValidationError>(py),
                "expected FmpValidationError, got {error}"
            );
            let value = error.value(py);
            let category: String = value
                .getattr("category")
                .expect("category attribute")
                .extract()
                .expect("category is a str");
            assert_eq!(category, "validation");
            assert!(value.getattr("endpoint").expect("endpoint").is_none());
            value.to_string()
        })
    }

    /// Runs `body` with an initialized interpreter attached.
    pub(crate) fn with_py<R>(body: impl FnOnce(Python<'_>) -> R) -> R {
        Python::initialize();
        Python::attach(body)
    }
}

#[cfg(test)]
mod tests {
    use libfmp::types::Limit;

    use super::*;

    fn limit(_name: &str, value: i64) -> PyResult<Limit> {
        u32::try_from(value)
            .map(Limit)
            .map_err(|_| crate::errors::validation_error("limit", "negative"))
    }

    #[test]
    fn optional_passes_none_through() {
        let converted = optional("limit", None::<i64>, limit).expect("none is fine");
        assert_eq!(converted, None);
    }

    #[test]
    fn optional_converts_some() {
        let converted = optional("limit", Some(5), limit).expect("valid limit");
        assert_eq!(converted, Some(Limit(5)));
    }

    #[test]
    fn optional_propagates_errors() {
        let error = optional("limit", Some(-1), limit).expect_err("negative rejected");
        assert_eq!(testing::validation_message(error), "limit: negative");
    }
}
