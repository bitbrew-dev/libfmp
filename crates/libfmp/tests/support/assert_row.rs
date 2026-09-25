//! Field-by-field assertions for `#[non_exhaustive]` response rows.
//!
//! Integration tests cannot build response structs with literals, so these
//! macros keep the literal-shaped expectations and check each listed field
//! against a decoded row. They also require the row to serialize to exactly
//! the listed number of fields, so a new response field still fails the test
//! until its expectation is added. An optional trailing format string names
//! the case in failure messages.

/// Asserts every listed field of one decoded row and that none is missing.
#[allow(unused_macros)] // Not every integration-test crate needs both macros.
macro_rules! assert_row {
    (@value $field:ident) => {
        $field
    };
    (@value $field:ident $value:expr) => {
        $value
    };
    (
        $actual:expr,
        $ty:ident { $($field:ident $(: $value:expr)?),* $(,)? }
        $(, $($context:tt)+)?
    ) => {{
        let context = String::new() $(+ &format!($($context)+))?;
        let actual = &$actual;
        let row: &$ty = ::std::borrow::Borrow::borrow(actual);
        $(
            assert_eq!(
                row.$field,
                assert_row!(@value $field $($value)?),
                "{}.{} {}",
                stringify!($ty),
                stringify!($field),
                context
            );
        )*
        let listed: &[&str] = &[$(stringify!($field)),*];
        let serialized = serde_json::to_value(row).unwrap();
        assert_eq!(
            serialized.as_object().map(serde_json::Map::len),
            Some(listed.len()),
            "{} has fields the expectation does not list {}",
            stringify!($ty),
            context
        );
    }};
}

/// Asserts a decoded row sequence against literal-shaped expectations in order.
#[allow(unused_macros)] // Not every integration-test crate needs both macros.
macro_rules! assert_rows {
    ($actual:expr, [$($ty:ident $body:tt),* $(,)?] $(, $($context:tt)+)?) => {{
        let context = String::new() $(+ &format!($($context)+))?;
        let rows = &$actual;
        let expected: &[&str] = &[$(stringify!($ty)),*];
        assert_eq!(rows.len(), expected.len(), "row count {}", context);
        let mut rows = rows.iter();
        $(assert_row!(rows.next().unwrap(), $ty $body, "{}", context);)*
    }};
}
