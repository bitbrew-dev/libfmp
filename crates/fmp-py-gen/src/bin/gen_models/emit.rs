//! Code emission: rendering each model struct, its conversions and getters, the
//! generated module tree, and the final `rustfmt` pass.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use fmp_py_gen::responses::{StructDef, Wrap, peel};

use crate::classify::classify;
use crate::dunder::{emit_dict_value_impl, emit_dunders};
use crate::model::{Class, KeptField, Registry, Transform};
use crate::{BoxError, GENERATED_HEADER, RUST_EDITION};

/// Emits one model struct, its `pymethods` impl, and its `From` conversion.
pub(crate) fn emit_struct(
    def: &StructDef,
    registry: &Registry,
    out: &mut String,
    report: &mut crate::model::Report,
) {
    let module_dotted = def.module_path.join(".");
    let source_path = public_source_path(&def.name, &def.module_path, registry);

    let mut kept: Vec<KeptField> = Vec::new();
    let mut skip_notes: Vec<String> = Vec::new();
    let mut secret_notes: Vec<String> = Vec::new();
    for field in &def.fields {
        let (wraps, base) = peel(&field.ty);
        let Some(base_ident) = base else {
            skip_notes.push(format!("{} (unsupported type shape)", field.name));
            report.unclassified.push(format!(
                "{}::{}: unsupported type shape",
                def.name, field.name
            ));
            continue;
        };
        match classify(&base_ident, registry, 0) {
            Class::Skip(reason) => {
                skip_notes.push(format!("{} ({reason})", field.name));
                report.unclassified.push(format!(
                    "{}::{}: {} ({reason})",
                    def.name, field.name, base_ident
                ));
            }
            Class::SecretUrl => {
                secret_notes.push(field.name.clone());
                report
                    .secret_fields
                    .push(format!("{}::{}", def.name, field.name));
                kept.push(KeptField::secret(field.name.clone(), wraps));
            }
            Class::Passthrough(pass) => {
                if wraps.len() > 1 || wraps.first() == Some(&Wrap::Vec) {
                    skip_notes.push(format!("{} (unsupported passthrough shape)", field.name));
                    report.unclassified.push(format!(
                        "{}::{}: passthrough under {:?}",
                        def.name,
                        field.name,
                        wrap_names(&wraps)
                    ));
                    continue;
                }
                let optional = wraps.first() == Some(&Wrap::Option);
                report.passthrough.insert(def.name.clone());
                if matches!(pass, crate::model::Pass::Number) {
                    report.number_fields += 1;
                }
                kept.push(KeptField::passthrough(field.name.clone(), pass, optional));
            }
            Class::Scalar {
                model_ty,
                transform,
            } => {
                kept.push(KeptField::scalar(
                    field.name.clone(),
                    wraps,
                    model_ty,
                    transform,
                    &mut report.enums,
                    &base_ident,
                ));
            }
        }
    }

    for note in &skip_notes {
        out.push_str(&format!(
            "/// NOTE: field `{note}` omitted from this model.\n"
        ));
    }
    for note in &secret_notes {
        out.push_str(&format!(
            "/// NOTE: field `{note}` is a secret URL: it is stored privately and is only \
             readable through its explicit `expose_secret_*` accessor.\n"
        ));
    }
    out.push_str("#[gen_stub_pyclass]\n");
    out.push_str(&format!(
        "#[pyclass(module = \"fmp._native.{module_dotted}\", frozen, eq, from_py_object)]\n"
    ));
    out.push_str("#[derive(Clone, PartialEq)]\n");
    out.push_str(&format!("pub(crate) struct {} {{\n", def.name));
    for field in &kept {
        out.push_str(&field.struct_field());
    }
    out.push_str("}\n\n");

    let needs_result = kept.iter().any(|field| field.is_passthrough());
    let params: Vec<String> = kept.iter().map(KeptField::new_param).collect();
    let signature: Vec<String> = kept.iter().map(|field| field.python_ident()).collect();

    out.push_str("#[gen_stub_pymethods]\n#[pymethods]\n");
    out.push_str(&format!("impl {} {{\n", def.name));
    out.push_str("    #[new]\n");
    out.push_str("    #[allow(clippy::too_many_arguments)]\n");
    out.push_str("    #[allow(clippy::fn_params_excessive_bools)]\n");
    out.push_str(&format!(
        "    #[pyo3(signature = (*, {}))]\n",
        signature.join(", ")
    ));
    let return_ty = if needs_result {
        "PyResult<Self>"
    } else {
        "Self"
    };
    out.push_str(&format!(
        "    fn new({}) -> {return_ty} {{\n",
        params.join(", ")
    ));
    for field in &kept {
        if let Some(stmt) = field.new_parse_stmt() {
            out.push_str(&format!("        {stmt}\n"));
        }
    }
    let assigns: Vec<String> = kept.iter().map(KeptField::new_assign).collect();
    let literal = format!("Self {{ {} }}", assigns.join(", "));
    if needs_result {
        out.push_str(&format!("        Ok({literal})\n"));
    } else {
        out.push_str(&format!("        {literal}\n"));
    }
    out.push_str("    }\n\n");

    emit_dunders(out, &def.name, &kept);
    for field in &kept {
        if let Some(getter) = field.getter_method() {
            out.push('\n');
            out.push_str(&getter);
        }
    }
    out.push_str("}\n\n");

    out.push_str(&format!("impl From<{source_path}> for {} {{\n", def.name));
    out.push_str(&format!("    fn from(value: {source_path}) -> Self {{\n"));
    out.push_str("        Self {\n");
    for field in &kept {
        out.push_str(&format!("            {},\n", field.conversion_assign()));
    }
    out.push_str("        }\n");
    out.push_str("    }\n}\n\n");
    emit_dict_value_impl(out, &def.name);
}

/// Resolves the public libfmp path of a struct, following `pub use` re-exports.
///
/// libfmp declares some submodules privately (`mod snapshots;`) while
/// re-exporting their structs from the parent, so the file-derived module path
/// is not always reachable. Private trailing segments are dropped until the path
/// consists solely of publicly declared modules.
fn public_source_path(name: &str, module_path: &[String], registry: &Registry) -> String {
    let mut public_path = module_path.to_vec();
    while let Some(child) = public_path.last().cloned() {
        let parent = public_path[..public_path.len() - 1].to_vec();
        let is_public = registry
            .module_public
            .get(&(parent, child))
            .copied()
            .unwrap_or(true);
        if is_public {
            break;
        }
        public_path.pop();
    }
    if public_path.is_empty() {
        format!("libfmp::responses::{name}")
    } else {
        format!("libfmp::responses::{}::{name}", public_path.join("::"))
    }
}

/// Builds the `From` conversion expression for a scalar field.
pub(crate) fn convert_expr(src: &str, wraps: &[Wrap], transform: &Transform) -> String {
    if matches!(transform, Transform::Identity) {
        return src.to_string();
    }
    match wraps.first() {
        None => base_transform(transform, src),
        Some(Wrap::Option) => {
            if wraps.len() == 1
                && let Some(function) = transform_fn(transform)
            {
                format!("{src}.map({function})")
            } else {
                format!(
                    "{src}.map(|value| {})",
                    convert_expr("value", &wraps[1..], transform)
                )
            }
        }
        Some(Wrap::Vec) => {
            if wraps.len() == 1
                && let Some(function) = transform_fn(transform)
            {
                format!("{src}.into_iter().map({function}).collect()")
            } else {
                format!(
                    "{src}.into_iter().map(|value| {}).collect()",
                    convert_expr("value", &wraps[1..], transform)
                )
            }
        }
    }
}

/// Returns a bare function path for transforms that forward their argument,
/// letting the emitted `map` avoid a redundant closure.
fn transform_fn(transform: &Transform) -> Option<String> {
    match transform {
        Transform::Nested(path) => Some(format!("{path}::from")),
        _ => None,
    }
}

/// Applies one scalar unwrap to an owned inner expression.
fn base_transform(transform: &Transform, inner: &str) -> String {
    match transform {
        Transform::Identity => inner.to_string(),
        Transform::IntoInner => format!("{inner}.into_inner()"),
        Transform::AsStrOwned => format!("{inner}.as_str().to_owned()"),
        Transform::Dot0 => format!("{inner}.0"),
        Transform::Get => format!("{inner}.get()"),
        Transform::ToString => format!("{inner}.to_string()"),
        Transform::ExposeSecret => format!("{inner}.expose_secret().to_owned()"),
        Transform::Nested(path) => format!("{path}::from({inner})"),
    }
}

/// Wraps a base model type in the field's `Option`/`Vec` layers.
pub(crate) fn wrap_type(wraps: &[Wrap], base: &str) -> String {
    match wraps.first() {
        None => base.to_string(),
        Some(Wrap::Option) => format!("Option<{}>", wrap_type(&wraps[1..], base)),
        Some(Wrap::Vec) => format!("Vec<{}>", wrap_type(&wraps[1..], base)),
    }
}

fn wrap_names(wraps: &[Wrap]) -> Vec<&'static str> {
    wraps
        .iter()
        .map(|wrap| match wrap {
            Wrap::Option => "Option",
            Wrap::Vec => "Vec",
        })
        .collect()
}

/// Computes the model file path mirroring a response module path.
pub(crate) fn model_file_path(root: &Path, module_path: &[String]) -> PathBuf {
    let mut path = root.to_path_buf();
    for (index, part) in module_path.iter().enumerate() {
        if index + 1 == module_path.len() {
            path.push(format!("{part}.rs"));
        } else {
            path.push(part);
        }
    }
    path
}

/// Writes a `mod.rs` for every directory in the generated model tree.
pub(crate) fn emit_module_tree(
    root: &Path,
    by_module: &BTreeMap<Vec<String>, Vec<&StructDef>>,
) -> Result<Vec<PathBuf>, BoxError> {
    let mut children: BTreeMap<Vec<String>, BTreeSet<String>> = BTreeMap::new();
    for module_path in by_module.keys() {
        for index in 0..module_path.len() {
            let parent = module_path[..index].to_vec();
            children
                .entry(parent)
                .or_default()
                .insert(module_path[index].clone());
        }
    }

    let mut written = Vec::new();
    for (dir_path, mods) in &children {
        let mut content = String::new();
        content.push_str(GENERATED_HEADER);
        content.push('\n');
        if dir_path.is_empty() {
            content.push_str("\npub(crate) mod convert;\n");
        }
        content.push('\n');
        for module in mods {
            content.push_str(&format!("pub mod {module};\n"));
        }
        let mut path = root.to_path_buf();
        for part in dir_path {
            path.push(part);
        }
        fs::create_dir_all(&path)?;
        path.push("mod.rs");
        fs::write(&path, content)?;
        written.push(path);
    }
    Ok(written)
}

/// Formats every generated file with `rustfmt`.
pub(crate) fn run_rustfmt(files: &[PathBuf]) -> Result<(), BoxError> {
    if files.is_empty() {
        return Ok(());
    }
    let mut command = Command::new("rustfmt");
    command.arg("--edition").arg(RUST_EDITION);
    for file in files {
        command.arg(file);
    }
    let status = command.status()?;
    if !status.success() {
        return Err(format!("rustfmt failed with status {status}").into());
    }
    Ok(())
}
