use std::fmt;

use pyo3::{
    create_exception,
    exceptions::{PyBaseException, PyException},
    prelude::*,
};

create_exception!(
    fmp.errors,
    FmpError,
    PyException,
    "Base exception for all fmp failures."
);
create_exception!(
    fmp.errors,
    FmpValidationError,
    FmpError,
    "An input failed local validation."
);
create_exception!(
    fmp.errors,
    FmpConfigError,
    FmpError,
    "Client or transport configuration is invalid."
);
create_exception!(
    fmp.errors,
    FmpTransportError,
    FmpError,
    "A request could not be completed by the transport."
);
create_exception!(
    fmp.errors,
    FmpStatusError,
    FmpError,
    "The provider returned a non-success HTTP status."
);
create_exception!(
    fmp.errors,
    FmpDecodeError,
    FmpError,
    "A successful response could not be decoded."
);

/// Convert a core error without exposing Rust implementation details or secrets.
///
/// Attribute assignment is fallible and is propagated as a Python exception;
/// this function never unwraps or panics at the FFI boundary.
pub(crate) fn to_py_error(error: libfmp::Error) -> PyErr {
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

/// Build a `FmpValidationError` for a Python argument that failed local
/// validation, naming the keyword so the caller can tell which input to fix.
///
/// The message reads `"{argument}: {reason}"` and the exception carries the
/// same attributes `to_py_error` sets on a libfmp validation error.
pub(crate) fn validation_error(argument: &str, reason: impl fmt::Display) -> PyErr {
    Python::attach(|py| {
        let exception = FmpValidationError::new_err(format!("{argument}: {reason}"));
        match set_validation_attributes(exception.value(py)) {
            Ok(()) => exception,
            Err(attribute_error) => attribute_error,
        }
    })
}

fn set_validation_attributes(exception: &Bound<'_, PyBaseException>) -> PyResult<()> {
    let py = exception.py();
    exception.setattr(
        "category",
        libfmp::error::ErrorCategory::Validation.as_str(),
    )?;
    exception.setattr("endpoint", py.None())?;
    exception.setattr("status", py.None())?;
    exception.setattr("body", py.None())?;
    exception.setattr("body_truncated", py.None())?;
    Ok(())
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

pub(crate) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = module.py();
    let base = py.get_type::<FmpError>();
    base.setattr("category", py.None())?;
    base.setattr("endpoint", py.None())?;
    base.setattr("status", py.None())?;
    base.setattr("body", py.None())?;
    base.setattr("body_truncated", py.None())?;

    py.get_type::<FmpValidationError>()
        .setattr("category", "validation")?;
    py.get_type::<FmpConfigError>()
        .setattr("category", "configuration")?;
    py.get_type::<FmpTransportError>()
        .setattr("category", "transport")?;
    py.get_type::<FmpStatusError>()
        .setattr("category", "status")?;
    py.get_type::<FmpDecodeError>()
        .setattr("category", "decode")?;

    module.add("_FmpError", base)?;
    module.add("_FmpValidationError", py.get_type::<FmpValidationError>())?;
    module.add("_FmpConfigError", py.get_type::<FmpConfigError>())?;
    module.add("_FmpTransportError", py.get_type::<FmpTransportError>())?;
    module.add("_FmpStatusError", py.get_type::<FmpStatusError>())?;
    module.add("_FmpDecodeError", py.get_type::<FmpDecodeError>())?;
    Ok(())
}
