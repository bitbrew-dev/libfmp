//! Conversion of dynamic `libfmp` JSON rows into native Python objects.
//!
//! Endpoints whose rows carry no typed model (`Vec<DynamicObject>`) hand each
//! row to Python as a plain `dict`. `serde_json` is compiled with
//! `arbitrary_precision`, so every number keeps the exact decimal text the
//! provider sent; this module walks the value tree explicitly and turns that
//! text into a Python `int` for any integer literal (however large) and into a
//! `float` for everything else, instead of routing through a generic
//! serializer that would only see the precision marker map.

use libfmp::codecs::{DynamicJson, DynamicObject};
use pyo3::IntoPyObjectExt;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

/// Converts one dynamic row into a Python `dict`, recursing into nested values.
pub fn dynamic_object_to_py<'py>(
    py: Python<'py>,
    object: &DynamicObject,
) -> PyResult<Bound<'py, PyAny>> {
    let dict = PyDict::new(py);
    for (name, value) in object {
        dict.set_item(name, dynamic_json_to_py(py, value)?)?;
    }
    Ok(dict.into_any())
}

/// Recursively converts an arbitrary JSON value into a native Python object:
/// object to `dict`, array to `list`, string to `str`, bool to `bool`, null to
/// `None`, integer literal to `int`, any other number to `float`.
pub fn dynamic_json_to_py<'py>(
    py: Python<'py>,
    value: &DynamicJson,
) -> PyResult<Bound<'py, PyAny>> {
    match value {
        DynamicJson::Null => Ok(py.None().into_bound(py)),
        DynamicJson::Bool(flag) => flag.into_bound_py_any(py),
        DynamicJson::Number(number) => number_to_py(py, number),
        DynamicJson::String(text) => text.into_bound_py_any(py),
        DynamicJson::Array(items) => {
            let list = PyList::empty(py);
            for item in items {
                list.append(dynamic_json_to_py(py, item)?)?;
            }
            Ok(list.into_any())
        }
        DynamicJson::Object(members) => dynamic_object_to_py(py, members),
    }
}

/// Converts a JSON number into a Python `int` when its literal is an integer
/// (exact at any magnitude) and into a `float` otherwise.
fn number_to_py<'py>(py: Python<'py>, number: &serde_json::Number) -> PyResult<Bound<'py, PyAny>> {
    if let Some(value) = number.as_u64() {
        return value.into_bound_py_any(py);
    }
    if let Some(value) = number.as_i64() {
        return value.into_bound_py_any(py);
    }
    let text = number.as_str();
    if is_integer_literal(text) {
        return py.import("builtins")?.getattr("int")?.call1((text,));
    }
    match number.as_f64() {
        Some(value) => value.into_bound_py_any(py),
        None => Err(PyValueError::new_err(format!(
            "JSON number is not representable as a Python int or float: {text}"
        ))),
    }
}

/// Whether `text` is an optional minus sign followed only by ASCII digits.
fn is_integer_literal(text: &str) -> bool {
    let digits = text.strip_prefix('-').unwrap_or(text);
    !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use pyo3::types::{PyBool, PyFloat, PyInt, PyString};

    use super::*;
    use crate::args::testing::with_py;

    fn parse(text: &str) -> DynamicJson {
        serde_json::from_str(text).expect("valid JSON")
    }

    fn convert<'py>(py: Python<'py>, text: &str) -> Bound<'py, PyAny> {
        dynamic_json_to_py(py, &parse(text)).expect("converts")
    }

    #[test]
    fn scalars_map_to_their_native_python_types() {
        with_py(|py| {
            assert!(convert(py, "null").is_none());
            let flag = convert(py, "true");
            assert!(flag.is_instance_of::<PyBool>());
            assert!(flag.extract::<bool>().expect("bool"));
            let text = convert(py, "\"AAPL\"");
            assert!(text.is_instance_of::<PyString>());
            assert_eq!(text.extract::<String>().expect("str"), "AAPL");
        });
    }

    #[test]
    fn integer_literals_become_exact_ints() {
        with_py(|py| {
            let small = convert(py, "42");
            assert!(small.is_instance_of::<PyInt>());
            assert_eq!(small.extract::<i64>().expect("int"), 42);
            let negative = convert(py, "-7");
            assert!(negative.is_instance_of::<PyInt>());
            assert_eq!(negative.extract::<i64>().expect("int"), -7);
            let big = convert(py, "123456789012345678901234567890");
            assert!(big.is_instance_of::<PyInt>());
            assert_eq!(
                big.str().expect("str").to_string(),
                "123456789012345678901234567890"
            );
            let big_negative = convert(py, "-98765432109876543210987654321");
            assert!(big_negative.is_instance_of::<PyInt>());
            assert_eq!(
                big_negative.str().expect("str").to_string(),
                "-98765432109876543210987654321"
            );
        });
    }

    #[test]
    fn non_integer_numbers_become_floats() {
        with_py(|py| {
            let fraction = convert(py, "1.5");
            assert!(fraction.is_instance_of::<PyFloat>());
            assert_eq!(fraction.extract::<f64>().expect("float"), 1.5);
            let exponent = convert(py, "1e3");
            assert!(exponent.is_instance_of::<PyFloat>());
            assert_eq!(exponent.extract::<f64>().expect("float"), 1000.0);
            let whole = convert(py, "2.0");
            assert!(whole.is_instance_of::<PyFloat>());
            assert_eq!(whole.extract::<f64>().expect("float"), 2.0);
        });
    }

    #[test]
    fn arrays_and_nested_objects_recurse() {
        with_py(|py| {
            let value = convert(
                py,
                r#"{"sicCode": "07371", "cik": 320193, "tags": [1, "x", null], "nested": {"deep": {"ok": true}}}"#,
            );
            let dict = value.cast::<PyDict>().expect("dict");
            assert_eq!(dict.len(), 4);
            let code: String = dict
                .get_item("sicCode")
                .expect("get")
                .expect("present")
                .extract()
                .expect("str");
            assert_eq!(code, "07371");
            let cik: i64 = dict
                .get_item("cik")
                .expect("get")
                .expect("present")
                .extract()
                .expect("int");
            assert_eq!(cik, 320_193);
            let tags = dict.get_item("tags").expect("get").expect("present");
            let tags = tags.cast::<PyList>().expect("list");
            assert_eq!(tags.len(), 3);
            assert_eq!(
                tags.get_item(0)
                    .expect("item")
                    .extract::<i64>()
                    .expect("int"),
                1
            );
            assert!(tags.get_item(2).expect("item").is_none());
            let nested = dict.get_item("nested").expect("get").expect("present");
            let deep = nested
                .cast::<PyDict>()
                .expect("dict")
                .get_item("deep")
                .expect("get")
                .expect("present");
            let ok: bool = deep
                .cast::<PyDict>()
                .expect("dict")
                .get_item("ok")
                .expect("get")
                .expect("present")
                .extract()
                .expect("bool");
            assert!(ok);
        });
    }

    #[test]
    fn object_helper_returns_a_dict_with_every_member() {
        with_py(|py| {
            let DynamicJson::Object(object) = parse(r#"{"a": 1, "b": [true]}"#) else {
                panic!("object expected");
            };
            let dict = dynamic_object_to_py(py, &object).expect("converts");
            let dict = dict.cast::<PyDict>().expect("dict");
            assert_eq!(dict.len(), 2);
            assert!(
                dict.get_item("b")
                    .expect("get")
                    .expect("present")
                    .is_instance_of::<PyList>()
            );
            let empty = dynamic_object_to_py(py, &DynamicObject::new()).expect("converts");
            assert_eq!(empty.cast::<PyDict>().expect("dict").len(), 0);
        });
    }

    #[test]
    fn integer_literal_detection_is_strict() {
        assert!(is_integer_literal("0"));
        assert!(is_integer_literal("-12"));
        assert!(!is_integer_literal(""));
        assert!(!is_integer_literal("-"));
        assert!(!is_integer_literal("1.0"));
        assert!(!is_integer_literal("1e3"));
    }
}
