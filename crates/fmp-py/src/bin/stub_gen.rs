//! Generates the `.pyi` type stubs for the `fmp` package from the annotated
//! pyclass models and client, driven by `pyo3-stub-gen`. Run with
//! `cargo run -p fmp-py --bin stub_gen`.
//!
//! pyo3-stub-gen emits `from . import <submodules>` in each package stub; this
//! binary rewrites those to absolute imports (`from fmp._native.<pkg> import
//! ...`) as a post-processing pass, since relative imports are disallowed here.

use std::path::{Path, PathBuf};

use pyo3_stub_gen::Result;

fn main() -> Result<()> {
    let stub = _native::stub_info()?;
    stub.generate()?;

    let python_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("python");
    absolutize_imports(&python_root, &python_root)?;
    Ok(())
}

/// Rewrites `from . import ...` to `from <package> import ...` in every
/// `__init__.pyi` under `dir`, where `<package>` is the file's dotted package
/// path relative to `root` (for example `fmp._native.bulk`).
fn absolutize_imports(root: &Path, dir: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            absolutize_imports(root, &path)?;
        } else if path.file_name().and_then(|name| name.to_str()) == Some("__init__.pyi") {
            let content = std::fs::read_to_string(&path)?;
            if !content.contains("from . import") {
                continue;
            }
            let package = package_path(root, &path);
            let rewritten = content.replace("from . import", &format!("from {package} import"));
            std::fs::write(&path, rewritten)?;
        }
    }
    Ok(())
}

/// Derives the dotted package path of an `__init__.pyi` from its parent
/// directory relative to `root` (for example `fmp._native.statements.growth`).
fn package_path(root: &Path, init_file: &Path) -> String {
    let parent = init_file.parent().unwrap_or(root);
    parent
        .strip_prefix(root)
        .unwrap_or(parent)
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join(".")
}
