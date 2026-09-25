//! Conversion of dynamic `serde_json` values held by generated models into
//! native Python objects.
//!
//! Passthrough fields (`serde_json::Value`, `serde_json::Map`, and
//! `serde_json::Number`) share the exact converter used for `dynamic` rows in
//! [`crate::convert`], so integer literals reach Python as exact `int`s at any
//! magnitude and non-integer numbers as `float`s on both paths.
//!
//! It also holds the shared pieces of the generated `__repr__` and
//! `to_dict()` methods.

use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3::types::PyList;

pub(crate) use crate::convert::{
    dynamic_json_to_py as json_to_py, dynamic_object_to_py as object_to_py, number_to_py,
};

/// A model field as `to_dict()` stores it: native values as-is, nested models
/// as their own `to_dict()`, recursively through `Option` and `Vec`.
pub(crate) trait DictValue {
    fn dict_value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>>;
}

macro_rules! native_dict_value {
    ($($ty:ty),+ $(,)?) => {
        $(impl DictValue for $ty {
            fn dict_value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
                self.clone().into_bound_py_any(py)
            }
        })+
    };
}

native_dict_value!(
    String,
    bool,
    f32,
    f64,
    i8,
    i16,
    i32,
    i64,
    isize,
    u8,
    u16,
    u32,
    u64,
    usize,
    chrono::NaiveDate,
    chrono::NaiveDateTime,
);

impl<T: DictValue> DictValue for Option<T> {
    fn dict_value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        match self {
            Some(value) => value.dict_value(py),
            None => Ok(py.None().into_bound(py)),
        }
    }
}

impl<T: DictValue> DictValue for Vec<T> {
    fn dict_value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let items = self
            .iter()
            .map(|item| item.dict_value(py))
            .collect::<PyResult<Vec<_>>>()?;
        Ok(PyList::new(py, items)?.into_any())
    }
}

/// Renders `Name(field=<repr>, ...)` from each field's Python `repr()`.
pub(crate) fn render_repr(name: &str, fields: &[(&str, Bound<'_, PyAny>)]) -> PyResult<String> {
    let mut parts = Vec::with_capacity(fields.len());
    for (field, value) in fields {
        parts.push(format!("{field}={}", value.repr()?.to_cow()?));
    }
    Ok(format!("{name}({})", parts.join(", ")))
}

#[cfg(test)]
mod tests {
    use pyo3::prelude::*;
    use pyo3::types::{PyDict, PyFloat, PyInt};

    use super::*;
    use crate::args::testing::with_py;

    fn parse(text: &str) -> serde_json::Value {
        serde_json::from_str(text).expect("valid JSON")
    }

    fn convert<'py>(py: Python<'py>, text: &str) -> Bound<'py, PyAny> {
        json_to_py(py, &parse(text)).expect("converts")
    }

    #[test]
    fn integers_beyond_u64_stay_exact_ints() {
        with_py(|py| {
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
    fn model_and_dynamic_paths_agree_on_every_number_shape() {
        with_py(|py| {
            for text in [
                "0",
                "42",
                "-7",
                "18446744073709551615",
                "-9223372036854775808",
                "123456789012345678901234567890",
                "1.5",
                "2.0",
                "1e3",
            ] {
                let value = parse(text);
                let model = json_to_py(py, &value).expect("model path converts");
                let dynamic =
                    crate::convert::dynamic_json_to_py(py, &value).expect("dynamic path converts");
                assert!(
                    model.get_type().is(dynamic.get_type()),
                    "type differs for {text}: {} vs {}",
                    model.get_type(),
                    dynamic.get_type()
                );
                assert!(
                    model.eq(&dynamic).expect("comparable"),
                    "value differs for {text}: {model} vs {dynamic}"
                );
            }
        });
    }

    #[test]
    fn existing_int_and_float_behaviour_is_unchanged() {
        with_py(|py| {
            let small = convert(py, "42");
            assert!(small.is_instance_of::<PyInt>());
            assert_eq!(small.extract::<i64>().expect("int"), 42);
            let max = convert(py, "18446744073709551615");
            assert!(max.is_instance_of::<PyInt>());
            assert_eq!(max.extract::<u64>().expect("int"), u64::MAX);
            let fraction = convert(py, "1.5");
            assert!(fraction.is_instance_of::<PyFloat>());
            assert_eq!(fraction.extract::<f64>().expect("float"), 1.5);
            let whole = convert(py, "2.0");
            assert!(whole.is_instance_of::<PyFloat>());
            let nested = convert(
                py,
                r#"{"pay": {"amount": "1", "components": [1, true, null]}}"#,
            );
            let dict = nested.cast::<PyDict>().expect("dict");
            let pay = dict.get_item("pay").expect("get").expect("present");
            assert_eq!(pay.cast::<PyDict>().expect("dict").len(), 2);
        });
    }
}
