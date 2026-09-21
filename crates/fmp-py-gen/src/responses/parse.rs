//! Source discovery and `syn` parsing: walking response files, deriving module
//! paths, and peeling wrapper types.

use std::fs;
use std::path::{Path, PathBuf};

use syn::{GenericArgument, Item, PathArguments, Type};

use super::DiscoverError;
use super::model::{FieldDef, StructDef, Wrap};

/// Peels `Option`/`Vec` wrappers and returns the base type identifier.
pub fn peel(ty: &Type) -> (Vec<Wrap>, Option<String>) {
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
pub fn base_ident(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(path) => path.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    }
}

/// Reports whether an item is `pub`.
pub fn is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

/// Reconstructs the technical-indicator structs hidden behind a declarative macro.
pub fn technical_indicator_structs(
    file: &syn::File,
    module_path: &[String],
) -> Result<Vec<StructDef>, DiscoverError> {
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
            return Err(DiscoverError::MacroArgs(tokens));
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
                ty: parse_type(field_ty)?,
            });
        }
        fields.push(FieldDef {
            name: metric,
            ty: parse_type(&metric_type)?,
        });
        defs.push(StructDef {
            name,
            module_path: module_path.to_vec(),
            fields,
        });
    }
    Ok(defs)
}

fn parse_type(text: &str) -> Result<Type, DiscoverError> {
    syn::parse_str::<Type>(text).map_err(|source| DiscoverError::MacroType {
        text: text.to_string(),
        source,
    })
}

/// Derives a module path from a response file's location.
pub fn module_path_for(file: &Path, root: &Path) -> Vec<String> {
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
pub fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), DiscoverError> {
    let entries = fs::read_dir(dir).map_err(|source| DiscoverError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let path = entry
            .map_err(|source| DiscoverError::Io {
                path: dir.to_path_buf(),
                source,
            })?
            .path();
        if path.is_dir() {
            collect_rust_files(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
    Ok(())
}
