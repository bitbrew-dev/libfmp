//! Rendering one endpoint: the `#[pymethods]` wrapper and the free
//! `<name>_query` builder it calls.
//!
//! The wrapper holds only the runtime hand-off (detach, `block_on`, result
//! mapping); argument conversion and query construction live in the builder
//! so an async twin can reuse them without touching this template. A
//! `binary = true` entry returns one `BinaryPayload` instead of a `Vec` of
//! models, with the same hand-off; a `response = "dynamic"` entry returns
//! each untyped row as a Python `dict` through `crate::convert`.

use std::fmt::Write as _;

use fmp_py_gen::registry::{Arg, CtorArg, Endpoint};

/// Clippy's `too_many_arguments` threshold.
const MAX_PARAMS: usize = 7;

/// The stub annotation for a dynamic-row method, whose Rust return type
/// (`Vec<Py<PyAny>>`) would otherwise stub as `list[typing.Any]`.
const DYNAMIC_STUB_OVERRIDE: &str = "    #[gen_stub(override_return_type(type_repr = \"list[dict[str, typing.Any]]\", imports = (\"typing\")))]";

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
    if endpoint.dynamic {
        out.push_str(DYNAMIC_STUB_OVERRIDE);
        out.push('\n');
    }
    if args.len() + 2 > MAX_PARAMS {
        out.push_str("    #[allow(clippy::too_many_arguments)]\n");
    }
    let (result, binding, mapping) = result_shape(endpoint);
    let mut decl = String::from("&self, py: Python<'_>");
    if !args.is_empty() {
        decl.push_str(", ");
        decl.push_str(&param_list(endpoint));
    }
    let _ = writeln!(out, "    fn {name}({decl}) -> PyResult<{result}> {{");
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
        "        let {binding} = py.detach(move || block_on(builder, |client| async move {{ {call} }}))?;"
    );
    let _ = writeln!(out, "        {mapping}");
    out.push_str("    }\n");
}

/// The Python return type, the local the detached call binds, and the
/// expression mapping that local into the return type.
fn result_shape(endpoint: &Endpoint) -> (String, &'static str, String) {
    if endpoint.binary {
        return (
            "BinaryPayload".to_owned(),
            "response",
            "response.map(BinaryPayload::from).map_err(to_py_error)".to_owned(),
        );
    }
    if endpoint.dynamic {
        return (
            "Vec<Py<PyAny>>".to_owned(),
            "rows",
            "rows.map_err(to_py_error)?.iter().map(|row| convert::dynamic_object_to_py(py, row).map(Bound::unbind)).collect()".to_owned(),
        );
    }
    let model = endpoint
        .response_model
        .as_ref()
        .map_or_else(|| "()".to_owned(), |model| model.name.clone());
    (
        format!("Vec<{model}>"),
        "rows",
        format!(
            "rows.map(|items| items.into_iter().map({model}::from).collect()).map_err(to_py_error)"
        ),
    )
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
    let ctor: Vec<String> = endpoint
        .ctor_args()
        .into_iter()
        .map(CtorArg::local)
        .collect();
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
