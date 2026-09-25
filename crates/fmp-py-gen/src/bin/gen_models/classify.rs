//! Base-type classification: mapping a libfmp newtype identifier to its
//! Python-model representation and unwrap transform.

use crate::model::{Class, Pass, Registry, Transform};

/// Classifies a base type identifier into its Python-model mapping.
pub(crate) fn classify(ident: &str, registry: &Registry, depth: usize) -> Class {
    if depth > 8 {
        return Class::Skip("alias cycle".to_string());
    }
    if let Some(prim) = primitive(ident) {
        return Class::Scalar {
            model_ty: prim.to_string(),
            transform: Transform::Identity,
        };
    }
    if let Some(pass) = passthrough(ident) {
        return Class::Passthrough(pass);
    }
    if string_into_inner(ident) {
        return Class::Scalar {
            model_ty: "String".to_string(),
            transform: Transform::IntoInner,
        };
    }
    if string_as_str(ident) {
        return Class::Scalar {
            model_ty: "String".to_string(),
            transform: Transform::AsStrOwned,
        };
    }
    if ident == "OpaqueDateText" {
        return Class::Scalar {
            model_ty: "String".to_string(),
            transform: Transform::Dot0,
        };
    }
    if ident == "Date" || ident == "UsDate" {
        return Class::Scalar {
            model_ty: "::chrono::NaiveDate".to_string(),
            transform: Transform::IntoInner,
        };
    }
    if ident == "ApiDateTime" {
        return Class::Scalar {
            model_ty: "::chrono::NaiveDateTime".to_string(),
            transform: Transform::IntoInner,
        };
    }
    if let Some((model_ty, transform)) = numeric(ident) {
        return Class::Scalar {
            model_ty: model_ty.to_string(),
            transform,
        };
    }
    if is_enum(ident) {
        return Class::Scalar {
            model_ty: "String".to_string(),
            transform: Transform::ToString,
        };
    }
    if let Some(module_path) = registry.structs.get(ident) {
        let path = format!("crate::models::{}::{ident}", module_path.join("::"));
        return Class::Scalar {
            model_ty: path.clone(),
            transform: Transform::Nested(path),
        };
    }
    if let Some(target) = registry.aliases.get(ident) {
        return classify(target, registry, depth + 1);
    }
    if ident == "SecretUrl" {
        return Class::SecretUrl;
    }
    Class::Skip("unclassified type".to_string())
}

fn primitive(ident: &str) -> Option<&'static str> {
    const PRIMS: &[&str] = &[
        "f64", "f32", "u64", "i64", "u32", "i32", "u16", "i16", "u8", "i8", "bool", "String",
        "usize", "isize",
    ];
    PRIMS.iter().copied().find(|prim| *prim == ident)
}

pub(crate) fn passthrough(ident: &str) -> Option<Pass> {
    match ident {
        "DynamicJson" | "Value" => Some(Pass::Value),
        "DynamicObject" => Some(Pass::Object),
        "Number" => Some(Pass::Number),
        _ => None,
    }
}

pub(crate) fn is_passthrough_name(ident: &str) -> bool {
    matches!(ident, "DynamicJson" | "DynamicObject")
}

fn string_into_inner(ident: &str) -> bool {
    const NAMES: &[&str] = &[
        "Ticker",
        "TipRanksExpertUid",
        "BulkPart",
        "SearchTerm",
        "Cik",
        "CongressionalMemberId",
        "FormType",
        "TransactionTypeCode",
        "Cusip",
        "Isin",
        "Lei",
        "ExchangeCode",
        "MarketHoursTimestamp",
        "CurrencyCode",
        "CountryCode",
        "Sector",
        "Industry",
        "BenchmarkYear",
        "NumericString",
    ];
    NAMES.contains(&ident)
}

fn string_as_str(ident: &str) -> bool {
    matches!(ident, "FiscalYearString" | "IsoTimestamp" | "PercentString")
}

fn numeric(ident: &str) -> Option<(&'static str, Transform)> {
    match ident {
        "CalendarYear" => Some(("u32", Transform::Get)),
        "CalendarQuarter" => Some(("u8", Transform::Get)),
        "PeriodLength" => Some(("u32", Transform::Get)),
        "FiniteDecimal" => Some(("f64", Transform::Get)),
        "UnixSeconds" | "UnixMilliseconds" => Some(("i64", Transform::Dot0)),
        "Page" | "Limit" | "Year" => Some(("u32", Transform::Dot0)),
        _ => None,
    }
}

/// The `typing.Literal` stub type of a closed enum field, listing every
/// spelling its `Display` produces; `None` for open or unlisted enums.
pub(crate) fn enum_literal(ident: &str) -> Option<&'static str> {
    match ident {
        "FiscalPeriod" => Some(r#"typing.Literal["Q1", "Q2", "Q3", "Q4", "FY"]"#),
        "YnFlag" => Some(r#"typing.Literal["Y", "N"]"#),
        "TitleCaseBoolFlag" => Some(r#"typing.Literal["True", "False"]"#),
        _ => None,
    }
}

fn is_enum(ident: &str) -> bool {
    const NAMES: &[&str] = &[
        "FiscalPeriod",
        "RetrievalFrequency",
        "StatementPeriod",
        "Quarter",
        "ChartTimeframe",
        "EconomicIndicator",
        "YnFlag",
        "YesNoFlag",
        "TrueFalseFlag",
        "TitleCaseBoolFlag",
        "DateOrDateTime",
    ];
    NAMES.contains(&ident)
}
