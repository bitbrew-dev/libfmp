//! Rendering one namespace node into Rust source.
//!
//! Every method follows the shape of the Phase 1 hand-written `QuoteNamespace`:
//! a free `<name>_query` function converts the Python arguments and builds
//! the `libfmp` query (the part an async twin will share), and the
//! `#[pymethods]` body only detaches from the interpreter, runs the call on
//! the shared runtime, and maps rows into the generated models.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::GENERATED_HEADER;
use crate::method::{render_method, render_query_fn};
use crate::plan::{Node, struct_name_for};

/// Where each verified entry's query type lives: dotted entry
/// (`statements.income.statement`) to segments under `libfmp::endpoints`.
pub(crate) type QueryModules = BTreeMap<String, Vec<String>>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum EmitError {
    #[error(
        "`{entry}`: query type `{query}` was not located under libfmp::endpoints (validation trusted it); the emitter needs its module path"
    )]
    UnresolvedQuery { entry: String, query: String },
}

/// One rendered namespace file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Rendered {
    pub(crate) source: String,
    /// Binary entries left out of the runtime path, as dotted entries.
    pub(crate) skipped: Vec<String>,
}

/// Renders `node` as the contents of its namespace file (unformatted).
pub(crate) fn render(node: &Node, query_modules: &QueryModules) -> Result<Rendered, EmitError> {
    let dotted = node.dotted();
    let struct_name = node.struct_name();
    let mut methods = Vec::new();
    let mut skipped = Vec::new();
    for endpoint in &node.endpoints {
        if endpoint.binary {
            skipped.push(format!("{dotted}.{}", endpoint.python_name));
        } else {
            methods.push(endpoint);
        }
    }

    let mut query_uses: BTreeMap<Vec<String>, BTreeSet<String>> = BTreeMap::new();
    let mut model_uses: BTreeMap<Vec<String>, BTreeSet<String>> = BTreeMap::new();
    for endpoint in &methods {
        let entry = format!("{dotted}.{}", endpoint.python_name);
        if let Some(query) = &endpoint.query_type {
            let module = query_modules
                .get(&entry)
                .ok_or_else(|| EmitError::UnresolvedQuery {
                    entry: entry.clone(),
                    query: query.clone(),
                })?;
            query_uses
                .entry(module.clone())
                .or_default()
                .insert(query.clone());
        }
        if let Some(model) = &endpoint.response_model {
            model_uses
                .entry(model.module.clone())
                .or_default()
                .insert(model.name.clone());
        }
    }

    let mut out = String::new();
    out.push_str(GENERATED_HEADER);
    out.push_str("\n\n");
    for child in &node.children {
        let _ = writeln!(out, "pub(crate) mod {child};");
    }
    if !node.children.is_empty() {
        out.push('\n');
    }

    out.push_str("use std::sync::Arc;\n\n");
    out.push_str("use libfmp::ClientBuilder;\n");
    for (module, names) in &query_uses {
        let _ = writeln!(out, "use {};", use_line("libfmp::endpoints", module, names));
    }
    out.push_str("use pyo3::prelude::*;\n");
    out.push_str("use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};\n\n");
    if methods.iter().any(|endpoint| !endpoint.args.is_empty()) {
        out.push_str("use crate::args;\n");
    }
    if !methods.is_empty() {
        out.push_str("use crate::errors::to_py_error;\n");
    }
    for (module, names) in &model_uses {
        let _ = writeln!(out, "use {};", use_line("crate::models", module, names));
    }
    if !methods.is_empty() {
        out.push_str("use crate::runtime::block_on;\n");
    }
    if !node.children.is_empty() {
        out.push('\n');
        for child in &node.children {
            let mut child_path = node.path.clone();
            child_path.push(child.clone());
            let _ = writeln!(out, "use self::{child}::{};", struct_name_for(&child_path));
        }
    }

    let _ = write!(
        out,
        "\n/// {} endpoints for a single client, exposed as `client.{dotted}`.\n",
        capitalize(&node.path.join(" "))
    );
    out.push_str("#[gen_stub_pyclass]\n");
    let _ = writeln!(out, "#[pyclass(module = \"fmp._native.{dotted}\", frozen)]");
    let _ = writeln!(
        out,
        "pub(crate) struct {struct_name} {{\n    builder: Arc<ClientBuilder>,\n}}\n"
    );
    let _ = writeln!(
        out,
        "impl {struct_name} {{\n    pub(crate) fn new(builder: Arc<ClientBuilder>) -> Self {{\n        Self {{ builder }}\n    }}\n}}\n"
    );

    out.push_str("#[gen_stub_pymethods]\n#[pymethods]\n");
    let _ = writeln!(out, "impl {struct_name} {{");
    let mut first = true;
    for endpoint in &methods {
        if !first {
            out.push('\n');
        }
        first = false;
        render_method(&mut out, endpoint);
    }
    for child in &node.children {
        if !first {
            out.push('\n');
        }
        first = false;
        let mut child_path = node.path.clone();
        child_path.push(child.clone());
        let child_struct = struct_name_for(&child_path);
        let _ = writeln!(
            out,
            "    /// The `{dotted}.{child}` endpoints, reached as `client.{dotted}.{child}`."
        );
        out.push_str("    #[getter]\n");
        let _ = writeln!(
            out,
            "    fn {child}(&self) -> {child_struct} {{\n        {child_struct}::new(self.builder.clone())\n    }}"
        );
    }
    out.push_str("}\n");

    for endpoint in &methods {
        if endpoint.query_type.is_some() {
            out.push('\n');
            render_query_fn(&mut out, endpoint, &struct_name);
        }
    }

    Ok(Rendered {
        source: out,
        skipped,
    })
}

/// `use <prefix>::<module>::{A, B};` with the braces dropped for one name.
fn use_line(prefix: &str, module: &[String], names: &BTreeSet<String>) -> String {
    let mut path = prefix.to_owned();
    for segment in module {
        path.push_str("::");
        path.push_str(segment);
    }
    let names: Vec<&str> = names.iter().map(String::as_str).collect();
    if let [single] = names.as_slice() {
        format!("{path}::{single}")
    } else {
        format!("{path}::{{{}}}", names.join(", "))
    }
}

fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}
