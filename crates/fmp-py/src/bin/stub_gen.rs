//! Generates the `.pyi` type stubs and the public `__init__.py` packages for
//! the `fmp` package from the annotated pyclass models and client, driven by
//! `pyo3-stub-gen`. Run with `cargo run -p fmp-py --bin stub_gen`.
//!
//! The raw stub output is post-processed so a run on an unchanged tree leaves
//! `git status` clean. Each `__init__.pyi` gets `from . import <submodules>`
//! rewritten to the absolute form (`from fmp._native.<pkg> import ...`, since
//! relative imports are disallowed here), loses the `# ruff: noqa` header
//! line, and is formatted with the pinned `ruff` in `.py` mode, which is the
//! layout the committed stubs use. The `.py` packages are written as-is.
//!
//! `ruff` [`RUFF_VERSION`] must be reachable: either `ruff` on `PATH` at that
//! exact version, or `uvx`, which fetches it on demand.

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use pyo3_stub_gen::Result;

/// The `ruff` release the committed stubs are formatted with; keep in step
/// with `ruff-pre-commit` in `crates/fmp-py/.pre-commit-config.yaml`.
const RUFF_VERSION: &str = "0.15.12";
const LINE_LENGTH: &str = "88";
const NOQA_PREFIX: &str = "# ruff: noqa";

fn main() -> Result<()> {
    let formatter = Formatter::locate()?;
    let stub = _native::stub_info()?;
    stub.generate()?;

    let python_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("python");
    postprocess_stubs(&python_root, &python_root, &formatter)?;
    Ok(())
}

/// The command that runs the pinned `ruff`.
struct Formatter {
    program: String,
    prefix: Vec<String>,
}

impl Formatter {
    /// Prefers `ruff` on `PATH` when it is exactly [`RUFF_VERSION`], then
    /// `uvx ruff@<version>`; fails naming both when neither works.
    fn locate() -> Result<Self> {
        let candidates = [
            Self {
                program: "ruff".to_owned(),
                prefix: Vec::new(),
            },
            Self {
                program: "uvx".to_owned(),
                prefix: vec![format!("ruff@{RUFF_VERSION}")],
            },
        ];
        for candidate in candidates {
            if candidate.reports_pinned_version() {
                return Ok(candidate);
            }
        }
        Err(std::io::Error::other(format!(
            "ruff {RUFF_VERSION} is required to format the stubs: install it on PATH \
             (`uv tool install ruff@{RUFF_VERSION}`) or make `uvx` available"
        ))
        .into())
    }

    fn reports_pinned_version(&self) -> bool {
        let output = Command::new(&self.program)
            .args(&self.prefix)
            .arg("--version")
            .stderr(Stdio::null())
            .output();
        match output {
            Ok(output) if output.status.success() => {
                String::from_utf8_lossy(&output.stdout).trim() == format!("ruff {RUFF_VERSION}")
            }
            _ => false,
        }
    }

    /// Formats `source` as a `.py` file at [`LINE_LENGTH`], ignoring any
    /// project configuration.
    fn format(&self, source: &str) -> Result<String> {
        let mut child = Command::new(&self.program)
            .args(&self.prefix)
            .args([
                "format",
                "--isolated",
                "--line-length",
                LINE_LENGTH,
                "--stdin-filename",
                "stub.py",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(source.as_bytes())?;
        }
        let output = child.wait_with_output()?;
        if !output.status.success() {
            return Err(std::io::Error::other(format!(
                "ruff format failed ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            ))
            .into());
        }
        Ok(String::from_utf8(output.stdout)?)
    }
}

/// Rewrites every `__init__.pyi` under `dir`: absolute imports, no `noqa`
/// header, pinned formatting.
fn postprocess_stubs(root: &Path, dir: &Path, formatter: &Formatter) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            postprocess_stubs(root, &path, formatter)?;
        } else if path.file_name().and_then(|name| name.to_str()) == Some("__init__.pyi") {
            let content = std::fs::read_to_string(&path)?;
            let package = package_path(root, &path);
            let rewritten =
                formatter.format(&strip_noqa(&absolutize_imports(&content, &package)))?;
            if rewritten != content {
                std::fs::write(&path, rewritten)?;
            }
        }
    }
    Ok(())
}

/// Rewrites `from . import ...` to `from <package> import ...`.
fn absolutize_imports(content: &str, package: &str) -> String {
    content.replace("from . import", &format!("from {package} import"))
}

/// Drops the `# ruff: noqa ...` header lines stub-gen writes.
fn strip_noqa(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    for line in content.split_inclusive('\n') {
        if !line.starts_with(NOQA_PREFIX) {
            out.push_str(line);
        }
    }
    out
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
