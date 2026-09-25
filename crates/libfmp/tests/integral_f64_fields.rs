//! Source scan: every response field typed with an integral-`f64` alias must
//! re-encode through `crate::codecs::integral_f64`.
//!
//! The aliases are plain `f64`, so the compiler cannot tell a field that
//! forgot the attribute from one that has it; without the attribute an
//! integral value re-encodes as `1.0` instead of `1`. The scan reads the
//! response sources as text so fields declared inside `macro_rules!` bodies
//! are covered too.

use std::{
    fs,
    path::{Path, PathBuf},
};

/// The `types` aliases whose response fields need the integral serializer.
/// Extend this list when another alias moves to `f64`.
const INTEGRAL_F64_ALIASES: &[&str] = &[
    "Volume",
    "MarketCapitalization",
    "TokenSupply",
    "SplitTerm",
    "MarketValue",
    "Quantity",
    "StatementAmount",
];

const SERIALIZE: &str = r#"serialize_with = "crate::codecs::integral_f64::serialize""#;
const SERIALIZE_OPTION: &str =
    r#"serialize_with = "crate::codecs::integral_f64::serialize_option""#;

fn rust_sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_sources(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

/// Returns the field type when `line` declares a struct field.
fn field_type(line: &str) -> Option<&str> {
    let rest = line.trim().strip_prefix("pub ")?;
    let (name, ty) = rest.split_once(':')?;
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return None;
    }
    Some(ty.trim().trim_end_matches(',').trim())
}

fn mentions(ty: &str, alias: &str) -> bool {
    ty.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .any(|token| token == alias)
}

/// Collects the `#[...]` attribute lines directly above `index`, skipping doc comments.
fn attributes_above(lines: &[&str], index: usize) -> Vec<String> {
    lines[..index]
        .iter()
        .rev()
        .map(|line| line.trim())
        .take_while(|line| line.starts_with("#[") || line.starts_with("///"))
        .filter(|line| line.starts_with("#["))
        .map(str::to_string)
        .collect()
}

fn violations() -> (usize, Vec<String>) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/responses");
    let mut files = Vec::new();
    rust_sources(&root, &mut files);
    files.sort();
    let mut checked = 0;
    let mut problems = Vec::new();
    for file in files {
        let source = fs::read_to_string(&file).unwrap();
        let lines: Vec<&str> = source.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let Some(ty) = field_type(line) else { continue };
            let Some(alias) = INTEGRAL_F64_ALIASES
                .iter()
                .find(|alias| mentions(ty, alias))
            else {
                continue;
            };
            checked += 1;
            let expected = if ty == *alias {
                SERIALIZE
            } else if ty == format!("Option<{alias}>") {
                SERIALIZE_OPTION
            } else {
                problems.push(format!(
                    "{}:{}: `{ty}` wraps `{alias}` in a shape the integral serializer does not cover",
                    file.display(),
                    index + 1
                ));
                continue;
            };
            if !attributes_above(&lines, index)
                .iter()
                .any(|attr| attr.contains(expected))
            {
                problems.push(format!(
                    "{}:{}: `{ty}` field lacks `#[serde({expected})]`",
                    file.display(),
                    index + 1
                ));
            }
        }
    }
    (checked, problems)
}

#[test]
fn every_integral_f64_alias_field_uses_the_integral_serializer() {
    let (checked, problems) = violations();
    assert!(checked > 0, "the scan found no integral-f64 alias fields");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn field_type_recognises_declarations_and_alias_tokens() {
    assert_eq!(field_type("    pub volume: Volume,"), Some("Volume"));
    assert_eq!(
        field_type("    pub market_cap: Option<MarketCapitalization>,"),
        Some("Option<MarketCapitalization>")
    );
    assert_eq!(field_type("pub use super::quote::Quote;"), None);
    assert_eq!(field_type("pub struct Quote {"), None);
    assert!(mentions("Option<Volume>", "Volume"));
    assert!(!mentions("VolumeWeighted", "Volume"));
    let lines = [
        "    /// Traded volume.",
        r#"    #[serde(serialize_with = "crate::codecs::integral_f64::serialize")]"#,
        "    pub volume: Volume,",
    ];
    assert_eq!(attributes_above(&lines, 2).len(), 1);
}
