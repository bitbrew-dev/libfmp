//! Rendering one endpoint: the `#[pymethods]` wrapper and the free
//! `<name>_query` builder it calls.
//!
//! The wrapper holds only the runtime hand-off (detach, `block_on`, row
//! mapping); argument conversion and query construction live in the builder
//! so an async twin can reuse them without touching this template.

use std::fmt::Write as _;

use fmp_py_gen::registry::{Arg, Endpoint};

/// Clippy's `too_many_arguments` threshold.
const MAX_PARAMS: usize = 7;

/// The Python parameters in signature order: required first, then optional.
fn params(endpoint: &Endpoint) -> Vec<&Arg> {
    let required = endpoint.args.iter().filter(|arg| arg.required);
    let optional = endpoint.args.iter().filter(|arg| !arg.required);
    required.chain(optional).collect()
}

/// `name: &str` or `name: Option<i64>`, with `args::` prefixed onto the
/// crate-defined input enums (`SymbolsArg`, `DateArg`, ...).
fn param_decl(arg: &Arg) -> String {
    let mut ty = String::new();
    let mut word = String::new();
    for c in arg.kind.input_type().chars().chain(std::iter::once(' ')) {
        if c.is_ascii_alphanumeric() || c == '_' {
            word.push(c);
            continue;
        }
        if word.starts_with(|w: char| w.is_ascii_uppercase()) {
            ty.push_str("args::");
        }
        ty.push_str(&word);
        word.clear();
        ty.push(c);
    }
    let ty = ty.trim_end();
    if arg.required {
        format!("{}: {ty}", arg.python_name())
    } else {
        format!("{}: Option<{ty}>", arg.python_name())
    }
}

fn param_list(endpoint: &Endpoint) -> String {
    params(endpoint)
        .iter()
        .map(|arg| param_decl(arg))
        .collect::<Vec<_>>()
        .join(", ")
}

fn name_list(endpoint: &Endpoint) -> String {
    params(endpoint)
        .iter()
        .map(|arg| arg.python_name())
        .collect::<Vec<_>>()
        .join(", ")
}

fn push_doc(out: &mut String, indent: &str, doc: &str) {
    for line in doc.trim().lines() {
        let line = line.trim_end();
        if line.is_empty() {
            let _ = writeln!(out, "{indent}///");
        } else {
            let _ = writeln!(out, "{indent}/// {line}");
        }
    }
}

pub(crate) fn render_method(out: &mut String, endpoint: &Endpoint) {
    let name = &endpoint.python_name;
    let args = params(endpoint);
    push_doc(out, "    ", &endpoint.doc);
    if !args.is_empty() {
        let mut signature: Vec<String> = Vec::new();
        for arg in args.iter().filter(|arg| arg.required) {
            signature.push(arg.python_name());
        }
        if args.iter().any(|arg| !arg.required) {
            signature.push("*".to_owned());
            for arg in args.iter().filter(|arg| !arg.required) {
                signature.push(format!("{}=None", arg.python_name()));
            }
        }
        let _ = writeln!(out, "    #[pyo3(signature = ({}))]", signature.join(", "));
    }
    if args.len() + 2 > MAX_PARAMS {
        out.push_str("    #[allow(clippy::too_many_arguments)]\n");
    }
    let model = endpoint
        .response_model
        .as_ref()
        .map_or_else(|| "()".to_owned(), |model| model.name.clone());
    let mut decl = String::from("&self, py: Python<'_>");
    if !args.is_empty() {
        decl.push_str(", ");
        decl.push_str(&param_list(endpoint));
    }
    let _ = writeln!(out, "    fn {name}({decl}) -> PyResult<Vec<{model}>> {{");
    let call = if endpoint.query_type.is_some() {
        let _ = writeln!(
            out,
            "        let query = {name}_query({})?;",
            name_list(endpoint)
        );
        format!("client.{}(query).await", endpoint.libfmp_method)
    } else {
        format!("client.{}().await", endpoint.libfmp_method)
    };
    out.push_str("        let builder = self.builder.clone();\n");
    let _ = writeln!(
        out,
        "        let rows = py.detach(move || block_on(builder, |client| async move {{ {call} }}))?;"
    );
    let _ = writeln!(
        out,
        "        rows.map(|items| items.into_iter().map({model}::from).collect()).map_err(to_py_error)"
    );
    out.push_str("    }\n");
}

pub(crate) fn render_query_fn(out: &mut String, endpoint: &Endpoint, struct_name: &str) {
    let name = &endpoint.python_name;
    let Some(query) = &endpoint.query_type else {
        return;
    };
    let _ = writeln!(
        out,
        "/// Builds the `{query}` for `{struct_name}::{name}` from validated Python arguments."
    );
    if endpoint.args.len() > MAX_PARAMS {
        out.push_str("#[allow(clippy::too_many_arguments)]\n");
    }
    let _ = writeln!(
        out,
        "fn {name}_query({}) -> PyResult<{query}> {{",
        param_list(endpoint)
    );
    for arg in params(endpoint) {
        let python = arg.python_name();
        let kind = arg.kind.name();
        if arg.required {
            let _ = writeln!(
                out,
                "    let {python} = args::{kind}(\"{python}\", {python})?;"
            );
        } else {
            let _ = writeln!(
                out,
                "    let {python} = args::optional(\"{python}\", {python}, args::{kind})?;"
            );
        }
    }
    let ctor: Vec<String> = endpoint.ctor_args().map(Arg::python_name).collect();
    if endpoint.setters.is_empty() {
        let _ = writeln!(out, "    Ok({query}::new({}))\n}}", ctor.join(", "));
        return;
    }
    let _ = writeln!(
        out,
        "    let mut query = {query}::new({});",
        ctor.join(", ")
    );
    for setter in &endpoint.setters {
        let Some(arg) = endpoint.setter_arg(setter) else {
            continue;
        };
        let python = arg.python_name();
        let _ = writeln!(
            out,
            "    if let Some({python}) = {python} {{\n        query = query.{}({python});\n    }}",
            setter.method
        );
    }
    out.push_str("    Ok(query)\n}\n");
}
