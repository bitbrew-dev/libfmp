//! The generated model inventory: every `pyclass` struct under
//! `crates/fmp-py/src/models/`, keyed by module path. The files are read back
//! with `syn` rather than recomputed from `libfmp`, so registration always
//! matches what `gen_models` last emitted.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use syn::Item;

/// Module path segments under `models/` to the sorted `pyclass` struct names
/// the module defines. Modules without a `pyclass` (`convert`) are absent.
pub(crate) type ModelModules = BTreeMap<Vec<String>, Vec<String>>;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ScanError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: syn::Error,
    },
}

/// Reads every `.rs` file under `root` and collects its `pyclass` structs.
pub(crate) fn scan_models(root: &Path) -> Result<ModelModules, ScanError> {
    let mut files = Vec::new();
    collect_rust_files(root, &mut files)?;
    files.sort();

    let mut modules = ModelModules::new();
    for file in &files {
        let content = fs::read_to_string(file).map_err(|source| ScanError::Io {
            path: file.clone(),
            source,
        })?;
        let parsed = syn::parse_file(&content).map_err(|source| ScanError::Parse {
            path: file.clone(),
            source,
        })?;
        let mut names: Vec<String> = parsed
            .items
            .iter()
            .filter_map(|item| match item {
                Item::Struct(item) if is_pyclass(item) => Some(item.ident.to_string()),
                _ => None,
            })
            .collect();
        if names.is_empty() {
            continue;
        }
        names.sort();
        modules
            .entry(module_path_for(root, file))
            .or_default()
            .extend(names);
    }
    Ok(modules)
}

fn is_pyclass(item: &syn::ItemStruct) -> bool {
    item.attrs
        .iter()
        .any(|attr| attr.path().is_ident("pyclass"))
}

/// `models/statements/growth/income.rs` -> `["statements", "growth", "income"]`;
/// a `mod.rs` names its directory.
pub(crate) fn module_path_for(root: &Path, file: &Path) -> Vec<String> {
    let relative = file.strip_prefix(root).unwrap_or(file);
    let mut segments: Vec<String> = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    if let Some(last) = segments.pop() {
        let stem = last.strip_suffix(".rs").unwrap_or(&last);
        if stem != "mod" {
            segments.push(stem.to_owned());
        }
    }
    segments
}

fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), ScanError> {
    let entries = fs::read_dir(dir).map_err(|source| ScanError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let path = entry
            .map_err(|source| ScanError::Io {
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
