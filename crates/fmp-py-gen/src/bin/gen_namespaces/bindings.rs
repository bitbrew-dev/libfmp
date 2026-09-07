//! The per-module wiring that used to be hand-written in `lib.rs`,
//! `client.rs`, and `facade.rs`: one binding per `fmp._native.<path>`
//! submodule, unioning the model inventory with the registry tree, rendered
//! as `registration.rs`, `namespaces/mod.rs`, `client_namespaces.rs`, and
//! `facade_domains.rs`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use crate::GENERATED_HEADER;
use crate::models::ModelModules;
use crate::plan::Node;

/// One `fmp._native.<path>` submodule and everything registered into it.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Binding {
    /// The path segments, `["bulk", "balance"]`.
    pub(crate) path: Vec<String>,
    /// The generated model structs the module holds, sorted.
    pub(crate) models: Vec<String>,
    /// The namespace struct name when the registry defines this path.
    pub(crate) namespace: Option<String>,
    /// The last path segment of every direct child, sorted.
    pub(crate) children: BTreeSet<String>,
}

impl Binding {
    /// The dotted Python path, `bulk.balance`.
    pub(crate) fn dotted(&self) -> String {
        self.path.join(".")
    }

    /// The registration function name, `register_bulk_balance`.
    fn function(&self) -> String {
        format!("register_{}", self.path.join("_"))
    }
}

/// Builds one binding per module path, plus the implicit parents.
pub(crate) fn build_bindings(
    models: &ModelModules,
    tree: &BTreeMap<Vec<String>, Node>,
) -> BTreeMap<Vec<String>, Binding> {
    let mut bindings: BTreeMap<Vec<String>, Binding> = BTreeMap::new();
    let paths = models.keys().chain(tree.keys());
    for path in paths {
        for depth in 1..path.len() {
            let parent = path[..depth].to_vec();
            bindings
                .entry(parent.clone())
                .or_insert_with(|| Binding {
                    path: parent,
                    ..Binding::default()
                })
                .children
                .insert(path[depth].clone());
        }
        let binding = bindings.entry(path.clone()).or_insert_with(|| Binding {
            path: path.clone(),
            ..Binding::default()
        });
        if let Some(names) = models.get(path) {
            binding.models = names.clone();
        }
        if let Some(node) = tree.get(path) {
            binding.namespace = Some(node.struct_name());
        }
    }
    bindings
}

/// Renders `registration.rs`: `register_namespaces` plus one function per
/// binding that builds the submodule and attaches it to its parent.
pub(crate) fn render_registration(bindings: &BTreeMap<Vec<String>, Binding>) -> String {
    let mut out = String::new();
    out.push_str(GENERATED_HEADER);
    out.push_str("\n\nuse pyo3::prelude::*;\n\nuse crate::add_submodule;\n\n");
    out.push_str(
        "/// Registers every `fmp._native.<domain>` submodule under the extension:\n\
         /// the generated models and namespace classes each one holds, nested for\n\
         /// the statements and bulk trees, each published in `sys.modules` so\n\
         /// `import fmp._native.<domain>` resolves.\n",
    );
    out.push_str(
        "pub(crate) fn register_namespaces(parent: &Bound<'_, PyModule>) -> PyResult<()> {\n",
    );
    for binding in bindings.values().filter(|binding| binding.path.len() == 1) {
        let _ = writeln!(out, "    {}(parent)?;", binding.function());
    }
    out.push_str("    Ok(())\n}\n");

    for binding in bindings.values() {
        let name = binding.path.last().cloned().unwrap_or_default();
        let _ = write!(
            out,
            "\nfn {}(parent: &Bound<'_, PyModule>) -> PyResult<()> {{\n    let module = PyModule::new(parent.py(), \"{name}\")?;\n",
            binding.function()
        );
        for model in &binding.models {
            let _ = writeln!(
                out,
                "    module.add_class::<crate::models::{}::{model}>()?;",
                binding.path.join("::")
            );
        }
        if let Some(namespace) = &binding.namespace {
            let _ = writeln!(
                out,
                "    module.add_class::<crate::namespaces::{}::{namespace}>()?;",
                binding.path.join("::")
            );
        }
        for child in &binding.children {
            let _ = writeln!(out, "    {}_{child}(&module)?;", binding.function());
        }
        let _ = writeln!(
            out,
            "    add_submodule(parent, \"fmp._native.{}\", &module)\n}}",
            binding.dotted()
        );
    }
    out
}

/// Renders `namespaces/mod.rs`: one module declaration per top-level domain.
pub(crate) fn render_namespaces_mod(tree: &BTreeMap<Vec<String>, Node>) -> String {
    let mut out = String::new();
    out.push_str(GENERATED_HEADER);
    out.push('\n');
    let domains: Vec<&Node> = tree.values().filter(|node| node.path.len() == 1).collect();
    if !domains.is_empty() {
        out.push('\n');
    }
    for node in domains {
        let _ = writeln!(out, "pub(crate) mod {};", node.dotted());
    }
    out
}

/// Renders `client_namespaces.rs`: the `FmpClient` getter for every domain.
pub(crate) fn render_client_namespaces(tree: &BTreeMap<Vec<String>, Node>) -> String {
    let domains: Vec<&Node> = tree.values().filter(|node| node.path.len() == 1).collect();
    let mut out = String::new();
    out.push_str(GENERATED_HEADER);
    out.push_str("\n\nuse pyo3::prelude::*;\nuse pyo3_stub_gen::derive::gen_stub_pymethods;\n\n");
    out.push_str("use crate::client::FmpClient;\n");
    for node in &domains {
        let _ = writeln!(
            out,
            "use crate::namespaces::{}::{};",
            node.dotted(),
            node.struct_name()
        );
    }
    out.push_str("\n#[gen_stub_pymethods]\n#[pymethods]\nimpl FmpClient {\n");
    let mut first = true;
    for node in &domains {
        if !first {
            out.push('\n');
        }
        first = false;
        let domain = node.dotted();
        let struct_name = node.struct_name();
        let _ = writeln!(
            out,
            "    /// The `{domain}` endpoints, reached as `client.{domain}`.\n    #[getter]\n    fn {domain}(&self) -> {struct_name} {{\n        {struct_name}::new(self.builder.clone())\n    }}"
        );
    }
    out.push_str("}\n");
    out
}

/// Renders `facade_domains.rs`: a wildcard `reexport_module_members!` from
/// every `fmp._native.<path>` submodule to its public `fmp.<path>` package.
pub(crate) fn render_facade_domains(bindings: &BTreeMap<Vec<String>, Binding>) -> String {
    let mut out = String::new();
    out.push_str(GENERATED_HEADER);
    out.push('\n');
    if !bindings.is_empty() {
        out.push('\n');
    }
    for binding in bindings.values() {
        let dotted = binding.dotted();
        let _ = writeln!(
            out,
            "pyo3_stub_gen::reexport_module_members!(\"fmp.{dotted}\" from \"fmp._native.{dotted}\");"
        );
    }
    out
}
