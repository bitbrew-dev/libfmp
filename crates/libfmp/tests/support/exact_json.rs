use serde_json::Value;

/// Compares decoded JSON while allowing harmless formatting changes made by
/// intentionally floating-point response fields.
///
/// Integer tokens outside JavaScript's exactly representable range are always
/// compared as [`serde_json::Number`] values so conversion to `f64` cannot hide
/// adjacent values or an integer-to-float kind change.
pub fn assert_json_wire_equivalent(actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Number(actual), Value::Number(expected)) => {
            const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

            let safely_comparable_as_f64 = |number: &serde_json::Number| {
                (number.is_f64() && number.as_f64().is_some())
                    || number
                        .as_i64()
                        .is_some_and(|value| value.unsigned_abs() <= MAX_SAFE_INTEGER)
                    || number
                        .as_u64()
                        .is_some_and(|value| value <= MAX_SAFE_INTEGER)
            };

            if actual == expected {
                return;
            }
            if (actual.is_f64() || expected.is_f64())
                && safely_comparable_as_f64(actual)
                && safely_comparable_as_f64(expected)
            {
                if let (Some(actual_float), Some(expected_float)) =
                    (actual.as_f64(), expected.as_f64())
                {
                    assert_eq!(actual_float, expected_float);
                } else {
                    assert_eq!(actual, expected);
                }
            } else {
                assert_eq!(actual, expected);
            }
        }
        (Value::Array(actual), Value::Array(expected)) => {
            assert_eq!(actual.len(), expected.len());
            for (actual, expected) in actual.iter().zip(expected) {
                assert_json_wire_equivalent(actual, expected);
            }
        }
        (Value::Object(actual), Value::Object(expected)) => {
            assert_eq!(actual.len(), expected.len());
            for (key, expected) in expected {
                let actual = actual
                    .get(key)
                    .unwrap_or_else(|| panic!("missing key {key}"));
                assert_json_wire_equivalent(actual, expected);
            }
        }
        _ => assert_eq!(actual, expected),
    }
}
