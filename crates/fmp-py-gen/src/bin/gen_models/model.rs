//! Python-side data types for the model generator: the classification
//! vocabulary, the surviving-field representation, and the run report.

use std::collections::{BTreeMap, BTreeSet};

use fmp_py_gen::responses::Wrap;

use crate::emit::{convert_expr, wrap_type};

/// The Python type a `serde_json::Number` field reads and accepts.
const NUMBER_STUB: &str = "builtins.int | builtins.float";

/// How a base (non-composed) field type maps into the Python model.
pub(crate) enum Class {
    Scalar {
        model_ty: String,
        transform: Transform,
    },
    Passthrough(Pass),
    SecretUrl,
    Skip(String),
}

/// The unwrapping applied to a scalar base value.
#[derive(Clone)]
pub(crate) enum Transform {
    Identity,
    IntoInner,
    AsStrOwned,
    Dot0,
    Get,
    ToString,
    ExposeSecret,
    Nested(String),
}

/// The dynamic-JSON shape of a serde_json passthrough field.
#[derive(Clone, Copy)]
pub(crate) enum Pass {
    Value,
    Object,
    Number,
}

impl Pass {
    pub(crate) fn raw_ty(self) -> &'static str {
        match self {
            Self::Value => "::serde_json::Value",
            Self::Object => "::serde_json::Map<String, ::serde_json::Value>",
            Self::Number => "::serde_json::Number",
        }
    }
}

/// Classification tables and the discovered struct and alias registries.
pub(crate) struct Registry {
    pub(crate) structs: BTreeMap<String, Vec<String>>,
    pub(crate) aliases: BTreeMap<String, String>,
    pub(crate) module_public: BTreeMap<(Vec<String>, String), bool>,
}

/// A field that survived classification and will appear in the model.
pub(crate) struct KeptField {
    source_name: String,
    field_ident: String,
    model_ty: String,
    kind: KeptKind,
}

enum KeptKind {
    Scalar {
        wraps: Vec<Wrap>,
        transform: Transform,
    },
    Passthrough {
        pass: Pass,
        optional: bool,
    },
    Secret {
        wraps: Vec<Wrap>,
    },
}

impl KeptField {
    pub(crate) fn scalar(
        name: String,
        wraps: Vec<Wrap>,
        base_model_ty: String,
        transform: Transform,
        enums: &mut BTreeSet<String>,
        base_ident: &str,
    ) -> Self {
        if matches!(transform, Transform::ToString) {
            enums.insert(base_ident.to_string());
        }
        let model_ty = wrap_type(&wraps, &base_model_ty);
        Self {
            field_ident: python_safe_ident(&name),
            source_name: name,
            model_ty,
            kind: KeptKind::Scalar { wraps, transform },
        }
    }

    pub(crate) fn passthrough(name: String, pass: Pass, optional: bool) -> Self {
        let raw = pass.raw_ty().to_string();
        let model_ty = if optional {
            format!("Option<{raw}>")
        } else {
            raw
        };
        Self {
            field_ident: python_safe_ident(&name),
            source_name: name,
            model_ty,
            kind: KeptKind::Passthrough { pass, optional },
        }
    }

    /// A `SecretUrl` field: stored as a crate-visible `String` with no
    /// `#[pyo3(get)]`, so only a hand-written accessor can read it.
    pub(crate) fn secret(name: String, wraps: Vec<Wrap>) -> Self {
        let model_ty = wrap_type(&wraps, "String");
        Self {
            field_ident: python_safe_ident(&name),
            source_name: name,
            model_ty,
            kind: KeptKind::Secret { wraps },
        }
    }

    pub(crate) fn is_passthrough(&self) -> bool {
        matches!(self.kind, KeptKind::Passthrough { .. })
    }

    pub(crate) fn is_secret(&self) -> bool {
        matches!(self.kind, KeptKind::Secret { .. })
    }

    /// The field as the Python object its attribute returns.
    pub(crate) fn python_value(&self) -> String {
        match &self.kind {
            KeptKind::Passthrough { .. } => format!("self.{}(py)?", self.field_ident),
            KeptKind::Scalar { .. } | KeptKind::Secret { .. } => {
                format!("self.{}.clone().into_bound_py_any(py)?", self.field_ident)
            }
        }
    }

    /// The field as `to_dict()` stores it, with nested models as dicts.
    pub(crate) fn dict_value(&self) -> String {
        match &self.kind {
            KeptKind::Passthrough { .. } => format!("self.{}(py)?", self.field_ident),
            KeptKind::Scalar { .. } | KeptKind::Secret { .. } => format!(
                "crate::models::convert::DictValue::dict_value(&self.{}, py)?",
                self.field_ident
            ),
        }
    }

    pub(crate) fn python_ident(&self) -> String {
        self.field_ident.clone()
    }

    pub(crate) fn struct_field(&self) -> String {
        match &self.kind {
            KeptKind::Passthrough { .. } => {
                format!("    {}: {},\n", self.field_ident, self.model_ty)
            }
            KeptKind::Secret { .. } => {
                format!("    pub(crate) {}: {},\n", self.field_ident, self.model_ty)
            }
            KeptKind::Scalar { .. } => {
                format!(
                    "    #[pyo3(get)]\n    pub {}: {},\n",
                    self.field_ident, self.model_ty
                )
            }
        }
    }

    pub(crate) fn new_param(&self) -> String {
        match &self.kind {
            KeptKind::Passthrough {
                pass: Pass::Number,
                optional,
            } => {
                let (stub, ty) = if *optional {
                    (format!("{NUMBER_STUB} | None"), "Option<Bound<'_, PyAny>>")
                } else {
                    (NUMBER_STUB.to_owned(), "Bound<'_, PyAny>")
                };
                format!(
                    "#[gen_stub(override_type(type_repr = \"{stub}\", imports = (\"builtins\",)))] {}: {ty}",
                    self.field_ident
                )
            }
            KeptKind::Passthrough { optional, .. } => {
                let ty = if *optional {
                    "Option<String>"
                } else {
                    "String"
                };
                format!("{}: {ty}", self.field_ident)
            }
            KeptKind::Scalar { .. } | KeptKind::Secret { .. } => {
                format!("{}: {}", self.field_ident, self.model_ty)
            }
        }
    }

    pub(crate) fn new_parse_stmt(&self) -> Option<String> {
        let KeptKind::Passthrough { pass, optional } = &self.kind else {
            return None;
        };
        if matches!(pass, Pass::Number) {
            let convert = "crate::models::convert::py_to_number";
            return Some(if *optional {
                format!(
                    "let {ident} = {ident}.as_ref().map({convert}).transpose()?;",
                    ident = self.field_ident
                )
            } else {
                format!(
                    "let {ident} = {convert}(&{ident})?;",
                    ident = self.field_ident
                )
            });
        }
        let inner = pass.raw_ty();
        let err = format!(
            ".map_err(|error| ::pyo3::exceptions::PyValueError::new_err(format!(\"invalid JSON for field `{}`: {{error}}\")))",
            self.field_ident
        );
        if *optional {
            Some(format!(
                "let {ident} = match {ident} {{ Some(text) => Some(::serde_json::from_str::<{inner}>(&text){err}?), None => None }};",
                ident = self.field_ident
            ))
        } else {
            Some(format!(
                "let {ident} = ::serde_json::from_str::<{inner}>(&{ident}){err}?;",
                ident = self.field_ident
            ))
        }
    }

    pub(crate) fn new_assign(&self) -> String {
        self.field_ident.clone()
    }

    /// The value `__getnewargs_ex__` passes back to `__new__` for this field.
    pub(crate) fn pickle_value(&self) -> String {
        match &self.kind {
            KeptKind::Passthrough {
                pass: Pass::Number, ..
            } => format!("self.{}(py)?", self.field_ident),
            KeptKind::Passthrough { optional, .. } => {
                let err = format!(
                    ".map_err(|error| ::pyo3::exceptions::PyValueError::new_err(format!(\"failed to serialize field `{}`: {{error}}\")))",
                    self.field_ident
                );
                if *optional {
                    format!(
                        "self.{name}.as_ref().map(::serde_json::to_string).transpose(){err}?.into_bound_py_any(py)?",
                        name = self.field_ident
                    )
                } else {
                    format!(
                        "::serde_json::to_string(&self.{name}){err}?.into_bound_py_any(py)?",
                        name = self.field_ident
                    )
                }
            }
            KeptKind::Scalar { .. } | KeptKind::Secret { .. } => {
                format!("self.{}.clone().into_bound_py_any(py)?", self.field_ident)
            }
        }
    }

    pub(crate) fn getter_method(&self) -> Option<String> {
        let KeptKind::Passthrough { pass, optional } = &self.kind else {
            return None;
        };
        let call = match pass {
            Pass::Value => "crate::models::convert::json_to_py",
            Pass::Object => "crate::models::convert::object_to_py",
            Pass::Number => "crate::models::convert::number_to_py",
        };
        let body = if *optional {
            format!(
                "        match &self.{name} {{\n            Some(value) => {call}(py, value),\n            None => Ok(py.None().into_bound(py)),\n        }}\n",
                name = self.field_ident
            )
        } else {
            format!(
                "        {call}(py, &self.{name})\n",
                name = self.field_ident
            )
        };
        let stub = match (pass, optional) {
            (Pass::Number, false) => format!(
                "    #[gen_stub(override_return_type(type_repr = \"{NUMBER_STUB}\", imports = (\"builtins\",)))]\n"
            ),
            (Pass::Number, true) => format!(
                "    #[gen_stub(override_return_type(type_repr = \"{NUMBER_STUB} | None\", imports = (\"builtins\",)))]\n"
            ),
            _ => String::new(),
        };
        Some(format!(
            "    #[getter]\n{stub}    fn {ident}<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {{\n{body}    }}\n",
            ident = self.field_ident
        ))
    }

    pub(crate) fn conversion_assign(&self) -> String {
        let src = format!("value.{}", self.source_name);
        let expr = match &self.kind {
            KeptKind::Passthrough { .. } => src,
            KeptKind::Scalar {
                wraps, transform, ..
            } => convert_expr(&src, wraps, transform),
            KeptKind::Secret { wraps } => convert_expr(&src, wraps, &Transform::ExposeSecret),
        };
        format!("{}: {expr}", self.field_ident)
    }
}

/// Accumulates the reporting facts printed at the end of a run.
#[derive(Default)]
pub(crate) struct Report {
    pub(crate) emitted: usize,
    pub(crate) passthrough: BTreeSet<String>,
    pub(crate) number_fields: usize,
    pub(crate) enums: BTreeSet<String>,
    pub(crate) secret_fields: Vec<String>,
    pub(crate) unclassified: Vec<String>,
}

impl Report {
    pub(crate) fn print(&self, discovered: &usize) {
        println!("gen_models summary");
        println!("  structs discovered: {discovered}");
        println!("  models emitted:     {}", self.emitted);
        println!("  enums -> str:       {:?}", self.enums);
        println!("  passthrough structs: {:?}", self.passthrough);
        println!("  serde_json::Number fields: {}", self.number_fields);
        println!("  SecretUrl fields (private): {:?}", self.secret_fields);
        println!("  unclassified (skipped): {:?}", self.unclassified);
    }
}

/// Maps a libfmp field name to a valid, non-reserved Python identifier.
///
/// The libfmp name may be a Rust raw identifier (`r#yield`); the `r#` prefix is
/// stripped first, then a trailing underscore is appended when the base collides
/// with a Python keyword or is not a valid Python identifier. The original name
/// is retained separately for libfmp field access.
pub(crate) fn python_safe_ident(name: &str) -> String {
    let base = name.strip_prefix("r#").unwrap_or(name);
    if is_python_keyword(base) || !is_valid_python_ident(base) {
        format!("{base}_")
    } else {
        base.to_string()
    }
}

/// Reports whether a name is a Python hard or soft keyword.
fn is_python_keyword(name: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
        "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
        "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return",
        "try", "while", "with", "yield", "match", "case", "type",
    ];
    KEYWORDS.contains(&name)
}

/// Reports whether a name is a syntactically valid Python identifier.
fn is_valid_python_ident(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first == '_' || first.is_ascii_alphabetic() => {}
        _ => return false,
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}
