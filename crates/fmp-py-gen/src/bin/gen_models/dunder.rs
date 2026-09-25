//! The Python protocol methods every model carries: keyword pickling, the
//! unhashable marker that pairs with the value `__eq__` from `#[pyclass(eq)]`,
//! `__match_args__`, a field-listing `__repr__`, and `to_dict()`.
//!
//! Secret fields (no getter) are left out of `__match_args__`, `__repr__`, and
//! `to_dict()`. A model with a secret field gets no generated `__repr__`: its
//! hand-written layer supplies a redacting one.

use crate::model::KeptField;

/// Appends the protocol methods to an open `#[pymethods]` block.
pub(crate) fn emit_dunders(out: &mut String, name: &str, kept: &[KeptField]) {
    let public: Vec<&KeptField> = kept.iter().filter(|field| !field.is_secret()).collect();
    emit_hash(out);
    emit_match_args(out, &public);
    emit_getnewargs_ex(out, kept);
    if public.len() == kept.len() {
        emit_repr(out, name, &public);
    }
    emit_to_dict(out, &public);
}

/// The `DictValue` impl that lets an enclosing model's `to_dict()` nest this
/// one as a dict.
pub(crate) fn emit_dict_value_impl(out: &mut String, name: &str) {
    out.push_str(&format!(
        "impl crate::models::convert::DictValue for {name} {{\n"
    ));
    out.push_str(
        "    fn dict_value<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {\n",
    );
    out.push_str("        Ok(self.to_dict(py)?.into_any())\n    }\n}\n\n");
}

/// The attribute names, in constructor order, typed as literal strings so a
/// type checker can resolve positional `case` patterns.
fn emit_match_args(out: &mut String, public: &[&KeptField]) {
    let names: Vec<String> = public
        .iter()
        .map(|field| format!("\"{}\"", field.python_ident()))
        .collect();
    let stub = if names.is_empty() {
        "tuple[()]".to_owned()
    } else {
        let literals: Vec<String> = names
            .iter()
            .map(|name| format!("typing.Literal[{}]", name.replace('"', "'")))
            .collect();
        format!("tuple[{}]", literals.join(", "))
    };
    out.push_str("    #[classattr]\n");
    out.push_str(&format!(
        "    #[gen_stub(override_return_type(type_repr = \"{stub}\", imports = (\"typing\",)))]\n"
    ));
    out.push_str("    fn __match_args__(py: Python<'_>) -> PyResult<Bound<'_, PyTuple>> {\n");
    out.push_str(&format!(
        "        PyTuple::new(py, [{}])\n    }}\n\n",
        names.join(", ")
    ));
}

/// `Name(field=repr(value), ...)` in constructor order.
fn emit_repr(out: &mut String, name: &str, public: &[&KeptField]) {
    out.push_str("    #[allow(clippy::clone_on_copy)]\n");
    out.push_str("    fn __repr__(&self, py: Python<'_>) -> PyResult<String> {\n");
    out.push_str("        crate::models::convert::render_repr(\n");
    out.push_str(&format!("            \"{name}\",\n"));
    out.push_str("            &[\n");
    for field in public {
        out.push_str(&format!(
            "                (\"{}\", {}),\n",
            field.python_ident(),
            field.python_value()
        ));
    }
    out.push_str("            ],\n        )\n    }\n\n");
}

/// The row as a `dict` keyed by attribute name, nested models included.
fn emit_to_dict(out: &mut String, public: &[&KeptField]) {
    out.push_str(
        "    /// Returns the row as a `dict` keyed by attribute name; nested models become\n",
    );
    out.push_str("    /// dicts and lists of models become lists of dicts.\n");
    out.push_str(
        "    #[gen_stub(override_return_type(type_repr = \"builtins.dict[builtins.str, typing.Any]\", imports = (\"builtins\", \"typing\")))]\n",
    );
    out.push_str("    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {\n");
    out.push_str("        let dict = PyDict::new(py);\n");
    for field in public {
        out.push_str(&format!(
            "        dict.set_item(\"{}\", {})?;\n",
            field.python_ident(),
            field.dict_value()
        ));
    }
    out.push_str("        Ok(dict)\n    }\n");
}

/// Value equality makes a row unhashable: `__hash__ = None`, as Python does
/// for a class that defines `__eq__` without `__hash__`.
fn emit_hash(out: &mut String) {
    out.push_str("    #[classattr]\n");
    out.push_str("    const __hash__: Option<Py<PyAny>> = None;\n\n");
}

/// Pickling calls the keyword-only `__new__` with every field by name.
fn emit_getnewargs_ex(out: &mut String, kept: &[KeptField]) {
    out.push_str("    #[allow(clippy::clone_on_copy)]\n");
    out.push_str(
        "    fn __getnewargs_ex__<'py>(&self, py: Python<'py>) -> PyResult<(Bound<'py, PyTuple>, Bound<'py, PyDict>)> {\n",
    );
    out.push_str("        let kwargs = PyDict::new(py);\n");
    for field in kept {
        out.push_str(&format!(
            "        kwargs.set_item(\"{}\", {})?;\n",
            field.python_ident(),
            field.pickle_value()
        ));
    }
    out.push_str("        Ok((PyTuple::empty(py), kwargs))\n");
    out.push_str("    }\n");
}
