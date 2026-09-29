//! Source scan: every response field typed with `Count` must decode through
//! `crate::codecs::count`.
//!
//! `Count` is a plain `u64`, so the compiler cannot tell a field that forgot
//! the attribute from one that has it; without the attribute an integral
//! float such as `3.0` fails the whole payload. The scan reads the response
//! sources as text so fields declared inside `macro_rules!` bodies are
//! covered too.

use std::{
    fs,
    path::{Path, PathBuf},
};

const DESERIALIZE: &str = r#"deserialize_with = "crate::codecs::count::deserialize""#;
const DESERIALIZE_OPTION: &str = r#"deserialize_with = "crate::codecs::count::deserialize_option""#;

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

fn mentions_count(ty: &str) -> bool {
    ty.split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
        .any(|token| token == "Count")
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
            if !mentions_count(ty) {
                continue;
            }
            checked += 1;
            let expected = match ty {
                "Count" => DESERIALIZE,
                "Option<Count>" => DESERIALIZE_OPTION,
                _ => {
                    problems.push(format!(
                        "{}:{}: `{ty}` wraps `Count` in a shape the count codec does not cover",
                        file.display(),
                        index + 1
                    ));
                    continue;
                }
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
fn every_count_field_uses_the_count_codec() {
    let (checked, problems) = violations();
    assert!(checked > 0, "the scan found no Count fields");
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
fn field_type_recognises_declarations_and_count_tokens() {
    assert_eq!(field_type("    pub buy: Count,"), Some("Count"));
    assert_eq!(
        field_type("    pub holders: Option<Count>,"),
        Some("Option<Count>")
    );
    assert_eq!(field_type("pub struct EmployeeCount {"), None);
    assert!(mentions_count("Option<Count>"));
    assert!(!mentions_count("EmployeeCount"));
    assert!(!mentions_count("CountryCode"));
    let lines = [
        "    /// Buy ratings.",
        r#"    #[serde(deserialize_with = "crate::codecs::count::deserialize")]"#,
        "    pub buy: Count,",
    ];
    assert_eq!(attributes_above(&lines, 2).len(), 1);
}
