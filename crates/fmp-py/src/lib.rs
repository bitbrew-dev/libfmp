//! Thin Python facade for `libfmp`.

mod client;
mod models;

use pyo3::{
    create_exception,
    exceptions::{PyBaseException, PyException, PyValueError},
    prelude::*,
};

create_exception!(
    fmp._native,
    FmpError,
    PyException,
    "Base exception for all fmp failures."
);
create_exception!(
    fmp._native,
    FmpValidationError,
    FmpError,
    "An input failed local validation."
);
create_exception!(
    fmp._native,
    FmpConfigError,
    FmpError,
    "Client or transport configuration is invalid."
);
create_exception!(
    fmp._native,
    FmpTransportError,
    FmpError,
    "A request could not be completed by the transport."
);
create_exception!(
    fmp._native,
    FmpStatusError,
    FmpError,
    "The provider returned a non-success HTTP status."
);
create_exception!(
    fmp._native,
    FmpDecodeError,
    FmpError,
    "A successful response could not be decoded."
);

/// Converts a core error without exposing Rust implementation details or secrets.
///
/// Attribute assignment is fallible and is propagated as a Python exception;
/// this function never unwraps or panics at the FFI boundary.
fn to_py_error(error: libfmp::Error) -> PyErr {
    Python::attach(|py| {
        let exception = match error.category() {
            libfmp::error::ErrorCategory::Validation => {
                FmpValidationError::new_err(error.to_string())
            }
            libfmp::error::ErrorCategory::Configuration => {
                FmpConfigError::new_err(error.to_string())
            }
            libfmp::error::ErrorCategory::Transport => {
                FmpTransportError::new_err(error.to_string())
            }
            libfmp::error::ErrorCategory::Status => FmpStatusError::new_err(error.to_string()),
            libfmp::error::ErrorCategory::Decode => FmpDecodeError::new_err(error.to_string()),
            _ => FmpError::new_err(error.to_string()),
        };

        match set_error_attributes(exception.value(py), &error) {
            Ok(()) => exception,
            Err(attribute_error) => attribute_error,
        }
    })
}

fn set_error_attributes(
    exception: &Bound<'_, PyBaseException>,
    error: &libfmp::Error,
) -> PyResult<()> {
    exception.setattr("category", error.category().as_str())?;
    exception.setattr("endpoint", error.endpoint())?;
    exception.setattr("status", error.status_code())?;
    exception.setattr("body", error.body().map(libfmp::error::SafeBody::as_str))?;
    exception.setattr(
        "body_truncated",
        error.body().map(libfmp::error::SafeBody::is_truncated),
    )?;
    Ok(())
}

#[pyfunction]
fn _test_error(category: &str) -> PyResult<()> {
    let secret = libfmp::error::SecretString::new("body-secret");
    let mut redactor = libfmp::error::Redactor::new();
    redactor.add_secret(&secret);
    let body = || libfmp::error::SafeBody::new("denied?apikey=query-secret body-secret", &redactor);
    let error = match category {
        "validation" => libfmp::Error::validation("invalid ticker"),
        "configuration" => libfmp::Error::configuration("invalid client configuration"),
        "transport" => libfmp::Error::transport(Some("quote-short"), "request failed"),
        "status" => libfmp::Error::status("quote-short", 401, Some(body())),
        "decode" => libfmp::Error::decode(
            Some("quote-short"),
            Some(200),
            Some(body()),
            "response decode failed",
        ),
        _ => return Err(PyValueError::new_err("unknown test error category")),
    };
    Err(to_py_error(error))
}

#[pymodule]
mod _native {
    use pyo3::prelude::*;

    #[pymodule_export]
    use super::{
        FmpConfigError, FmpDecodeError, FmpError, FmpStatusError, FmpTransportError,
        FmpValidationError,
    };

    #[pymodule_export]
    use super::client::FmpClient;

    #[pymodule_export]
    use super::models::QuoteShort;

    #[pymodule_init]
    fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add("__version__", libfmp::VERSION)?;
        module.add_function(wrap_pyfunction!(super::_test_error, module)?)?;

        let py = module.py();
        let base = py.get_type::<super::FmpError>();
        base.setattr("category", py.None())?;
        base.setattr("endpoint", py.None())?;
        base.setattr("status", py.None())?;
        base.setattr("body", py.None())?;
        base.setattr("body_truncated", py.None())?;

        py.get_type::<super::FmpValidationError>()
            .setattr("category", "validation")?;
        py.get_type::<super::FmpConfigError>()
            .setattr("category", "configuration")?;
        py.get_type::<super::FmpTransportError>()
            .setattr("category", "transport")?;
        py.get_type::<super::FmpStatusError>()
            .setattr("category", "status")?;
        py.get_type::<super::FmpDecodeError>()
            .setattr("category", "decode")?;
        Ok(())
    }
}
