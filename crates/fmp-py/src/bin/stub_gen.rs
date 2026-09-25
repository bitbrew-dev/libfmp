//! Generates the `.pyi` type stubs and the public `__init__.py` packages for
//! the `fmp` package from the annotated pyclass models and client, driven by
//! `pyo3-stub-gen`. Run with `cargo run -p fmp-py --bin stub_gen`.
//!
//! The raw stub output is post-processed so a run on an unchanged tree leaves
//! `git status` clean. Each `__init__.pyi` gets `from . import <submodules>`
//! rewritten to the absolute form (`from fmp._native.<pkg> import ...`, since
//! relative imports are disallowed here), gets every module reference in an
//! annotation fully qualified (a method named like an imported module, such
//! as `ForexNamespace.quote`, would otherwise shadow it inside the class
//! body), loses the `# ruff: noqa` header line, and is formatted with the pinned `ruff` in `.py` mode, which is the
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
const UNHASHABLE_STUB: &str = "__hash__: typing.Optional[typing.Any] = None";
const UNHASHABLE_TYPESHED: &str = "__hash__: typing.ClassVar[None]  # type: ignore[assignment]";

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
            let rewritten = formatter.format(&strip_noqa(&mark_unhashable(
                &qualify_module_references(&absolutize_imports(&content, &package)),
            )))?;
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

/// Qualifies every `module.Name` reference to a module imported with
/// `from <package> import <module>`, and adds the matching
/// `import <package>.<module>` line.
///
/// A class body that defines a method named like an imported module rebinds
/// that name for the rest of the class, so a later `-> list[quote.Quote]`
/// resolves `quote` to the method. The fully qualified `fmp._native.quote`
/// path cannot be shadowed that way. The original `from` import is kept so
/// submodule names listed in `__all__` stay bound. Docstrings are left as-is.
fn qualify_module_references(content: &str) -> String {
    let modules: Vec<(String, String)> = content
        .lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("from ")?;
            let (package, name) = rest.split_once(" import ")?;
            let is_ident =
                !name.is_empty() && name.chars().all(|c| c == '_' || c.is_ascii_alphanumeric());
            is_ident.then(|| (package.to_owned(), name.to_owned()))
        })
        .collect();
    if modules.is_empty() {
        return content.to_owned();
    }
    let mut out = String::with_capacity(content.len());
    let mut in_docstring = false;
    for line in content.split_inclusive('\n') {
        let quotes = line.matches("\"\"\"").count();
        if in_docstring || quotes > 0 {
            if quotes % 2 == 1 {
                in_docstring = !in_docstring;
            }
            out.push_str(line);
            continue;
        }
        if line.starts_with("from ") || line.starts_with("import ") {
            out.push_str(line);
            if let Some((package, name)) = modules
                .iter()
                .find(|(package, name)| line.trim_end() == format!("from {package} import {name}"))
            {
                out.push_str(&format!("import {package}.{name}\n"));
            }
            continue;
        }
        let mut qualified = line.to_owned();
        for (package, name) in &modules {
            qualified = qualify(&qualified, name, &format!("{package}.{name}"));
        }
        out.push_str(&qualified);
    }
    out
}

/// Replaces each `name.` that starts a dotted reference (not preceded by an
/// identifier character or a dot) with `qualified.`.
fn qualify(line: &str, name: &str, qualified: &str) -> String {
    let needle = format!("{name}.");
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(index) = rest.find(&needle) {
        out.push_str(&rest[..index]);
        let starts_reference = out
            .chars()
            .next_back()
            .is_none_or(|c| !(c == '_' || c == '.' || c.is_ascii_alphanumeric()));
        if starts_reference {
            out.push_str(qualified);
            out.push('.');
        } else {
            out.push_str(&needle);
        }
        rest = &rest[index + needle.len()..];
    }
    out.push_str(rest);
    out
}

/// Rewrites the `__hash__ = None` class attribute of the value-equality models
/// to the typeshed spelling, so the stub itself type-checks and pyright rejects
/// a model used as a `set` member or `dict` key (mypy does not check that).
fn mark_unhashable(content: &str) -> String {
    content.replace(UNHASHABLE_STUB, UNHASHABLE_TYPESHED)
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

#[cfg(test)]
mod tests {
    use super::{mark_unhashable, qualify_module_references};

    #[test]
    fn unhashable_models_use_the_typeshed_spelling() {
        let stub = "class Row:\n    __hash__: typing.Optional[typing.Any] = None\n";
        let expected =
            "class Row:\n    __hash__: typing.ClassVar[None]  # type: ignore[assignment]\n";
        assert_eq!(mark_unhashable(stub), expected);
    }

    #[test]
    fn method_named_like_a_module_cannot_shadow_it() {
        let stub = "import builtins\nfrom fmp._native import quote\n\nclass ForexNamespace:\n    def quote(self) -> builtins.list[quote.Quote]:\n        r\"\"\"\n        Returns a quote.Quote row.\n        \"\"\"\n    def quote_short(self) -> builtins.list[quote.QuoteShort]: ...\n";
        let expected = "import builtins\nfrom fmp._native import quote\nimport fmp._native.quote\n\nclass ForexNamespace:\n    def quote(self) -> builtins.list[fmp._native.quote.Quote]:\n        r\"\"\"\n        Returns a quote.Quote row.\n        \"\"\"\n    def quote_short(self) -> builtins.list[fmp._native.quote.QuoteShort]: ...\n";
        assert_eq!(qualify_module_references(stub), expected);
    }

    #[test]
    fn only_whole_references_are_qualified() {
        let stub =
            "from fmp._native import chart\nx: sub_chart.Bar\ny: a.chart.Bar\nz: chart.Bar\n";
        let expected = "from fmp._native import chart\nimport fmp._native.chart\nx: sub_chart.Bar\ny: a.chart.Bar\nz: fmp._native.chart.Bar\n";
        assert_eq!(qualify_module_references(stub), expected);
    }

    #[test]
    fn stubs_without_module_imports_are_unchanged() {
        let stub = "import builtins\nimport typing\n\nclass A: ...\n";
        assert_eq!(qualify_module_references(stub), stub);
    }
}
