//! Hand-written layer over the generated `FinancialReportDate` model.
//!
//! `FinancialReportDate.link_json` / `link_xlsx` are `libfmp::error::SecretUrl`
//! values: the download URLs embed the API key, so libfmp redacts them in
//! `Debug` and `Display`. `gen_models` stores them as crate-private `String`
//! fields with no `#[pyo3(get)]`; this module is the only reader.
//!
//! Reading a URL is conspicuous (`expose_secret_url_json()`), mirroring the
//! `danger_allow_insecure_authentication` pattern on `FmpClient`. `repr()`
//! and `str()` print `[REDACTED URL]` unless the process-wide reveal flag is
//! on, which is set by `set_reveal_secret_urls(True)` or, at first use, by the
//! `FMP_REVEAL_SECRET_URLS` environment variable. Pickles carry the real URLs
//! because `__getnewargs__` must round-trip the value.

use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

use pyo3::prelude::*;
use pyo3::types::PyString;
use pyo3_stub_gen::derive::{gen_stub_pyfunction, gen_stub_pymethods};

use crate::models::statements::reports::FinancialReportDate;

/// Environment variable consulted once, at first use of the reveal flag.
pub(crate) const REVEAL_ENV: &str = "FMP_REVEAL_SECRET_URLS";

/// Placeholder printed in place of a secret URL while the flag is off.
const REDACTED: &str = "[REDACTED URL]";

static REVEAL: LazyLock<AtomicBool> = LazyLock::new(|| {
    let value = std::env::var_os(REVEAL_ENV);
    AtomicBool::new(reveal_from_env(
        value.as_deref().and_then(|value| value.to_str()),
    ))
});

/// Interprets the environment value: `1`, `true`, or `yes` (case-insensitive,
/// surrounding whitespace ignored) enable revealing; anything else, including
/// an unset variable, leaves it off.
fn reveal_from_env(value: Option<&str>) -> bool {
    value.is_some_and(|value| {
        matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes"
        )
    })
}

/// Turns revealing of secret URLs in `repr()` / `str()` on or off for the
/// whole process.
///
/// Off by default. Enable only for local debugging: while on, every
/// `FinancialReportDate` repr prints download links that embed the API key.
#[gen_stub_pyfunction(module = "fmp._native")]
#[pyfunction]
pub(crate) fn set_reveal_secret_urls(enabled: bool) {
    REVEAL.store(enabled, Ordering::SeqCst);
}

/// Reports whether secret URLs are currently revealed in `repr()` / `str()`.
#[gen_stub_pyfunction(module = "fmp._native")]
#[pyfunction]
pub(crate) fn reveal_secret_urls() -> bool {
    REVEAL.load(Ordering::SeqCst)
}

#[gen_stub_pymethods]
#[pymethods]
impl FinancialReportDate {
    /// Returns the JSON report download URL. The URL embeds the API key:
    /// treat the value as a credential and never log it.
    fn expose_secret_url_json(&self) -> String {
        self.link_json.clone()
    }

    /// Returns the XLSX report download URL. The URL embeds the API key:
    /// treat the value as a credential and never log it.
    fn expose_secret_url_xlsx(&self) -> String {
        self.link_xlsx.clone()
    }

    /// Prints every field; the secret URLs show as `[REDACTED URL]` unless
    /// `set_reveal_secret_urls(True)` (or `FMP_REVEAL_SECRET_URLS`) is on.
    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {
        Ok(format!(
            "FinancialReportDate(symbol={}, fiscal_year={}, period={}, link_json={}, link_xlsx={})",
            python_repr(py, &self.symbol)?,
            self.fiscal_year,
            python_repr(py, &self.period)?,
            secret_repr(py, &self.link_json)?,
            secret_repr(py, &self.link_xlsx)?,
        ))
    }
}

/// Renders `value` the way Python's `repr()` renders a `str`.
fn python_repr(py: Python<'_>, value: &str) -> PyResult<String> {
    Ok(PyString::new(py, value).repr()?.to_cow()?.into_owned())
}

/// Renders a secret URL: the placeholder, or its Python repr when revealed.
fn secret_repr(py: Python<'_>, value: &str) -> PyResult<String> {
    if reveal_secret_urls() {
        python_repr(py, value)
    } else {
        Ok(REDACTED.to_owned())
    }
}

/// Adds the reveal toggle functions to `fmp._native`.
pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(set_reveal_secret_urls, module)?)?;
    module.add_function(wrap_pyfunction!(reveal_secret_urls, module)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::reveal_from_env;

    #[test]
    fn truthy_values_enable_revealing() {
        for value in ["1", "true", "TRUE", "True", "yes", "YES", " yes ", "\t1\n"] {
            assert!(reveal_from_env(Some(value)), "{value:?} should reveal");
        }
    }

    #[test]
    fn other_values_keep_redaction() {
        for value in ["", "0", "false", "no", "on", "y", "enabled", "1 1"] {
            assert!(!reveal_from_env(Some(value)), "{value:?} should redact");
        }
    }

    #[test]
    fn unset_variable_keeps_redaction() {
        assert!(!reveal_from_env(None));
    }
}
