//! Python-facing spellings of registry names.
//!
//! A registry arg is named after the `libfmp` setter it feeds (`from` for
//! `with_from`), which can collide with a Python keyword. The emitter and
//! the stubs use the sanitized spelling from [`python_safe_ident`]; the
//! setter name is untouched. This mirrors the rule `gen_models` applies to
//! response field names.

/// Returns `name` with a trailing `_` when it is a Python hard or soft
/// keyword, after stripping any Rust raw-identifier prefix.
pub fn python_safe_ident(name: &str) -> String {
    let base = name.strip_prefix("r#").unwrap_or(name);
    if is_python_keyword(base) {
        format!("{base}_")
    } else {
        base.to_owned()
    }
}

/// Reports whether `name` is a Python hard or soft keyword.
pub fn is_python_keyword(name: &str) -> bool {
    const KEYWORDS: &[&str] = &[
        "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
        "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
        "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return",
        "try", "while", "with", "yield", "match", "case", "type",
    ];
    KEYWORDS.contains(&name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_get_a_trailing_underscore() {
        assert_eq!(python_safe_ident("from"), "from_");
        assert_eq!(python_safe_ident("type"), "type_");
        assert_eq!(python_safe_ident("match"), "match_");
        assert_eq!(python_safe_ident("r#type"), "type_");
    }

    #[test]
    fn plain_names_pass_through() {
        assert_eq!(python_safe_ident("symbol"), "symbol");
        assert_eq!(python_safe_ident("to_date"), "to_date");
        assert_eq!(python_safe_ident("r#symbol"), "symbol");
    }
}
