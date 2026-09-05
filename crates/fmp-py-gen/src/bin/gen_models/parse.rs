//! Source discovery and `syn` parsing: walking response files, deriving module
//! paths, peeling wrapper types, and sanitizing field names for Python.

use std::fs;
use std::path::{Path, PathBuf};

use syn::{GenericArgument, Item, PathArguments, Type};

use crate::BoxError;
use crate::model::{FieldDef, StructDef, Wrap};

/// Peels `Option`/`Vec` wrappers and returns the base type identifier.
pub(crate) fn peel(ty: &Type) -> (Vec<Wrap>, Option<String>) {
    let mut wraps = Vec::new();
    let mut current = ty;
    loop {
        let Type::Path(path) = current else {
            return (wraps, None);
        };
        let Some(segment) = path.path.segments.last() else {
            return (wraps, None);
        };
        let ident = segment.ident.to_string();
        if (ident == "Option" || ident == "Vec")
            && let PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(GenericArgument::Type(inner)) = args.args.first()
        {
            wraps.push(if ident == "Option" {
                Wrap::Option
            } else {
                Wrap::Vec
            });
            current = inner;
            continue;
        }
        return (wraps, Some(ident));
    }
}

/// Returns the last path-segment identifier of a type, ignoring generics.
pub(crate) fn base_ident(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(path) => path.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    }
}

pub(crate) fn is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

/// Reconstructs the technical-indicator structs hidden behind a declarative macro.
pub(crate) fn technical_indicator_structs(
    file: &syn::File,
    module_path: &[String],
) -> Result<Vec<StructDef>, BoxError> {
    let mut defs = Vec::new();
    for item in &file.items {
        let Item::Macro(item) = item else { continue };
        if item.mac.path.segments.last().map(|s| s.ident.to_string())
            != Some("technical_indicator_row".to_string())
        {
            continue;
        }
        let tokens = item.mac.tokens.to_string();
        let parts: Vec<String> = tokens
            .split(',')
            .map(|part| part.trim().to_string())
            .collect();
        if parts.len() != 3 {
            return Err(format!("unexpected technical_indicator_row args: {tokens}").into());
        }
        let name = parts[0].clone();
        let metric = parts[1].clone();
        let metric_type = parts[2].clone();
        let mut fields = Vec::new();
        for (field_name, field_ty) in [
            ("date", "ApiDateTime"),
            ("open", "Price"),
            ("high", "Price"),
            ("low", "Price"),
            ("close", "Price"),
            ("volume", "Volume"),
        ] {
            fields.push(FieldDef {
                name: field_name.to_string(),
                ty: syn::parse_str::<Type>(field_ty)?,
            });
        }
        fields.push(FieldDef {
            name: metric,
            ty: syn::parse_str::<Type>(&metric_type)?,
        });
        defs.push(StructDef {
            name,
            module_path: module_path.to_vec(),
            fields,
        });
    }
    Ok(defs)
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

/// Derives a module path from a response file's location.
pub(crate) fn module_path_for(file: &Path, root: &Path) -> Vec<String> {
    let rel = file.strip_prefix(root).unwrap_or(file);
    let mut comps: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if let Some(last) = comps.pop() {
        let stem = last.strip_suffix(".rs").unwrap_or(&last).to_string();
        if stem != "mod" {
            comps.push(stem);
        }
    }
    comps
}

/// Recursively collects every `.rs` file under a directory.
pub(crate) fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), BoxError> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    Ok(())
}
