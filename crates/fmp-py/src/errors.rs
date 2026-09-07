//! The `fmp.errors` exception hierarchy.
//!
//! Each exception is created with `pyo3::create_exception!` under the module
//! `fmp.errors`, so `__module__`, `repr`, and pickle all point at the public
//! package. The stub inventory entry is submitted separately under
//! `fmp._native.errors`, the runtime submodule that `register` builds and that
//! the generated `fmp/errors/__init__.py` re-exports from.
//! `pyo3_stub_gen::create_exception!` cannot express this split (it uses one
//! module for both), renders user-defined bases as `builtins.<Base>`, and has
//! no way to declare the structured attributes, hence the local macro.

use std::fmt;

use pyo3::{
    create_exception,
    exceptions::{PyBaseException, PyException},
    prelude::*,
};
use pyo3_stub_gen::{
    PyStubType, TypeInfo,
    type_info::{MemberInfo, PyClassInfo},
};

/// The native submodule whose stub declares the exception classes.
const NATIVE_MODULE: &str = "fmp._native.errors";

/// The structured attributes every exception carries, typed for the stub.
///
/// `to_py_error` sets them per instance; `register` sets class-level defaults
/// so bare instances (`FmpStatusError("message")`) expose the same names.
static ERROR_ATTRIBUTES: &[MemberInfo] = &[
    MemberInfo {
        name: "category",
        r#type: <Option<String> as PyStubType>::type_output,
        doc: "The error category, for example `\"status\"`; `None` on the base class.",
        default: None,
        deprecated: None,
    },
    MemberInfo {
        name: "endpoint",
        r#type: <Option<String> as PyStubType>::type_output,
        doc: "The endpoint path the failing request targeted, when known.",
        default: None,
        deprecated: None,
    },
    MemberInfo {
        name: "status",
        r#type: <Option<u16> as PyStubType>::type_output,
        doc: "The HTTP status code the provider returned, when known.",
        default: None,
        deprecated: None,
    },
    MemberInfo {
        name: "body",
        r#type: <Option<String> as PyStubType>::type_output,
        doc: "A redacted, size-bounded excerpt of the response body, when known.",
        default: None,
        deprecated: None,
    },
    MemberInfo {
        name: "body_truncated",
        r#type: <Option<bool> as PyStubType>::type_output,
        doc: "Whether `body` was cut short to stay within the excerpt bound.",
        default: None,
        deprecated: None,
    },
];

/// Declares one exception type: the pyo3 type object under `fmp.errors` plus
/// the `pyo3-stub-gen` inventory entry under `fmp._native.errors`. Only the
/// base class lists the attributes; subclasses inherit them in the stub.
macro_rules! fmp_exception {
    ($name:ident, $base:ty, $doc:expr) => {
        fmp_exception!($name, $base, $doc, &[]);
    };
    ($name:ident, $base:ty, $doc:expr, $getters:expr) => {
        create_exception!(fmp.errors, $name, $base, $doc);

        impl PyStubType for $name {
            fn type_output() -> TypeInfo {
                TypeInfo::locally_defined(stringify!($name), NATIVE_MODULE.into())
            }
        }

        pyo3_stub_gen::impl_py_runtime_type!($name);

        pyo3_stub_gen::inventory::submit! {
            PyClassInfo {
                pyclass_name: stringify!($name),
                struct_id: std::any::TypeId::of::<$name>,
                getters: $getters,
                setters: &[],
                module: Some(NATIVE_MODULE),
                doc: $doc,
                bases: &[<$base as PyStubType>::type_output],
                has_eq: false,
                has_ord: false,
                has_hash: false,
                has_str: false,
                subclass: true,
            }
        }
    };
}

fmp_exception!(
    FmpError,
    PyException,
    "Base exception for all fmp failures.",
    ERROR_ATTRIBUTES
);
fmp_exception!(
    FmpValidationError,
    FmpError,
    "An input failed local validation."
);
fmp_exception!(
    FmpConfigError,
    FmpError,
    "Client or transport configuration is invalid."
);
fmp_exception!(
    FmpTransportError,
    FmpError,
    "A request could not be completed by the transport."
);
fmp_exception!(
    FmpStatusError,
    FmpError,
    "The provider returned a non-success HTTP status."
);
fmp_exception!(
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

/// Publishes the hierarchy under `fmp._native.errors` (public names, the
/// module the generated `fmp.errors` package re-exports from) and, for one
/// release, under the private `_Fmp*` names directly on `fmp._native`.
///
/// Class-level attribute defaults make a bare instance report its category
/// and no request context, matching what `to_py_error` sets per instance.
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

    let errors = PyModule::new(py, "errors")?;
    errors.add("FmpError", &base)?;
    errors.add("FmpValidationError", py.get_type::<FmpValidationError>())?;
    errors.add("FmpConfigError", py.get_type::<FmpConfigError>())?;
    errors.add("FmpTransportError", py.get_type::<FmpTransportError>())?;
    errors.add("FmpStatusError", py.get_type::<FmpStatusError>())?;
    errors.add("FmpDecodeError", py.get_type::<FmpDecodeError>())?;
    super::add_submodule(module, NATIVE_MODULE, &errors)?;

    module.add("_FmpError", base)?;
    module.add("_FmpValidationError", py.get_type::<FmpValidationError>())?;
    module.add("_FmpConfigError", py.get_type::<FmpConfigError>())?;
    module.add("_FmpTransportError", py.get_type::<FmpTransportError>())?;
    module.add("_FmpStatusError", py.get_type::<FmpStatusError>())?;
    module.add("_FmpDecodeError", py.get_type::<FmpDecodeError>())?;
    Ok(())
}
