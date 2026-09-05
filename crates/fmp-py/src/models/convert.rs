//! Runtime conversion of dynamic `serde_json` values into native Python objects.
//!
//! `serde_json` is compiled with `arbitrary_precision`, under which a foreign
//! serializer such as `pythonize` observes each number as an internal
//! `{"$serde_json::private::Number": "..."}` marker map rather than a scalar.
//! These helpers therefore walk the value tree explicitly instead of routing it
//! through a generic serializer, preserving the integer or float distinction the
//! provider sent.

use pyo3::IntoPyObjectExt;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Converts a JSON number into a Python `int` or `float`, preserving its kind.
pub(crate) fn number_to_py<'py>(
    py: Python<'py>,
    number: &serde_json::Number,
) -> PyResult<Bound<'py, PyAny>> {
    if let Some(value) = number.as_u64() {
        value.into_bound_py_any(py)
    } else if let Some(value) = number.as_i64() {
        value.into_bound_py_any(py)
    } else if let Some(value) = number.as_f64() {
        value.into_bound_py_any(py)
    } else {
        Err(PyValueError::new_err(format!(
            "JSON number is not representable as a Python int or float: {number}"
        )))
    }
}

/// Recursively converts an arbitrary JSON value into a native Python object.
pub(crate) fn json_to_py<'py>(
    py: Python<'py>,
    value: &serde_json::Value,
) -> PyResult<Bound<'py, PyAny>> {
    match value {
        serde_json::Value::Null => Ok(py.None().into_bound(py)),
        serde_json::Value::Bool(value) => value.into_bound_py_any(py),
        serde_json::Value::Number(number) => number_to_py(py, number),
        serde_json::Value::String(value) => value.into_bound_py_any(py),
        serde_json::Value::Array(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(json_to_py(py, item)?)?;
            }
            Ok(list.into_any())
        }
        serde_json::Value::Object(members) => object_to_py(py, members),
    }
}

/// Converts a JSON object into a native Python `dict`.
pub(crate) fn object_to_py<'py>(
    py: Python<'py>,
    members: &serde_json::Map<String, serde_json::Value>,
) -> PyResult<Bound<'py, PyAny>> {
    let dict = PyDict::new(py);
    for (name, value) in members {
        dict.set_item(name, json_to_py(py, value)?)?;
    }
    Ok(dict.into_any())
}
