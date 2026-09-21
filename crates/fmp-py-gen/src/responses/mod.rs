//! The shared reader of the `libfmp` response structs.
//!
//! [`discover`] walks `crates/libfmp/src/responses`, parses every file with
//! `syn`, and returns the language-neutral [`Discovery`]: the response
//! structs in source order, the public type aliases, and the visibility of
//! each declared submodule. Emitters (`gen_models` today, `gen_go` next)
//! classify and render from this one reader so they never disagree about
//! what libfmp exposes.

pub mod model;
pub mod parse;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use syn::{Fields, Item};

pub use model::{FieldAttrs, FieldDef, StructDef, Wrap, apply_rename_rule};
pub use parse::{
    base_ident, collect_rust_files, field_attrs, is_public, module_path_for, peel,
    struct_rename_all, technical_indicator_structs,
};

/// Everything an emitter needs to know about the response surface.
#[derive(Debug, Default)]
pub struct Discovery {
    /// Every public named-field struct, files sorted by path, source order
    /// within a file (macro-generated technical-indicator rows first).
    pub structs: Vec<StructDef>,
    /// Public `type Alias = Target;` declarations, keyed by alias name. A
    /// response-file alias takes precedence over one declared in the crate's
    /// `types.rs`, `codecs.rs`, or `query.rs`.
    pub aliases: BTreeMap<String, String>,
    /// Whether each `mod` item declared inside a response file is `pub`,
    /// keyed by the declaring module path and the submodule name.
    pub module_public: BTreeMap<(Vec<String>, String), bool>,
}

/// Why the response tree could not be read.
#[derive(Debug, thiserror::Error)]
pub enum DiscoverError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: {source}")]
    Syntax {
        path: PathBuf,
        #[source]
        source: syn::Error,
    },
    #[error("unexpected technical_indicator_row args: {0}")]
    MacroArgs(String),
    #[error("invalid technical_indicator_row type `{text}`: {source}")]
    MacroType {
        text: String,
        #[source]
        source: syn::Error,
    },
    #[error("{path}: {item}: {source}")]
    Attribute {
        path: PathBuf,
        item: String,
        #[source]
        source: syn::Error,
    },
    #[error("responses directory {0} has no parent")]
    NoParent(PathBuf),
}

/// Parses every response struct under `responses_root`.
pub fn discover(responses_root: &Path) -> Result<Discovery, DiscoverError> {
    let mut files = Vec::new();
    collect_rust_files(responses_root, &mut files)?;
    files.sort();

    let mut discovery = Discovery::default();
    for file in &files {
        let module_path = module_path_for(file, responses_root);
        let parsed = parse_file(file)?;
        let stem = module_path.last().cloned().unwrap_or_default();
        if stem == "technical_indicators" {
            discovery
                .structs
                .extend(technical_indicator_structs(&parsed, &module_path)?);
        }
        for item in &parsed.items {
            match item {
                Item::Mod(item) => {
                    discovery.module_public.insert(
                        (module_path.clone(), item.ident.to_string()),
                        is_public(&item.vis),
                    );
                }
                Item::Struct(item) if is_public(&item.vis) => {
                    if let Fields::Named(named) = &item.fields {
                        let name = item.ident.to_string();
                        let attribute_error = |source| DiscoverError::Attribute {
                            path: file.clone(),
                            item: name.clone(),
                            source,
                        };
                        let rename_all = struct_rename_all(&item.attrs).map_err(attribute_error)?;
                        let mut fields = Vec::with_capacity(named.named.len());
                        for field in &named.named {
                            let Some(ident) = &field.ident else { continue };
                            fields.push(FieldDef {
                                name: ident.to_string(),
                                ty: field.ty.clone(),
                                attrs: field_attrs(&field.attrs).map_err(attribute_error)?,
                            });
                        }
                        discovery.structs.push(StructDef {
                            name,
                            module_path: module_path.clone(),
                            rename_all,
                            fields,
                        });
                    }
                }
                Item::Type(item) if is_public(&item.vis) => {
                    if let Some(target) = base_ident(&item.ty) {
                        discovery.aliases.insert(item.ident.to_string(), target);
                    }
                }
                _ => {}
            }
        }
    }

    let src_root = responses_root
        .parent()
        .ok_or_else(|| DiscoverError::NoParent(responses_root.to_path_buf()))?;
    for definition in ["types.rs", "codecs.rs", "query.rs"] {
        let parsed = parse_file(&src_root.join(definition))?;
        for item in &parsed.items {
            if let Item::Type(item) = item
                && is_public(&item.vis)
                && let Some(target) = base_ident(&item.ty)
            {
                discovery
                    .aliases
                    .entry(item.ident.to_string())
                    .or_insert(target);
            }
        }
    }

    Ok(discovery)
}

fn parse_file(path: &Path) -> Result<syn::File, DiscoverError> {
    let content = fs::read_to_string(path).map_err(|source| DiscoverError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    syn::parse_file(&content).map_err(|source| DiscoverError::Syntax {
        path: path.to_path_buf(),
        source,
    })
}
