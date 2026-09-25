//! The Python protocol methods every model carries: keyword pickling and the
//! unhashable marker that pairs with the value `__eq__` from `#[pyclass(eq)]`.

use crate::model::KeptField;

/// Appends the protocol methods to an open `#[pymethods]` block.
pub(crate) fn emit_dunders(out: &mut String, kept: &[KeptField]) {
    emit_hash(out);
    emit_getnewargs_ex(out, kept);
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
