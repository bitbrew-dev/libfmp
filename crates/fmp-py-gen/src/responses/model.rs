//! The language-neutral intermediate representation of a discovered response
//! struct: its name, module path, serde container rule, and named fields with
//! their `syn` types and serde field attributes.

use syn::Type;

/// A single response struct discovered in libfmp.
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub module_path: Vec<String>,
    /// The first paragraph of the struct's `///` doc comment, lines joined
    /// with one space, if any.
    pub doc: Option<String>,
    /// The container-level `#[serde(rename_all = "...")]` rule, if any.
    pub rename_all: Option<String>,
    pub fields: Vec<FieldDef>,
    /// Whether the same file carries a hand-written
    /// `impl<'de> Deserialize<'de> for <name>` instead of a derive, so the
    /// field list alone does not describe the wire shape.
    pub custom_deserialize: bool,
}

/// One named field of a response struct.
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: String,
    pub ty: Type,
    pub attrs: FieldAttrs,
}

/// The serde facts captured from a field's `#[serde(...)]` attributes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FieldAttrs {
    /// `rename = "..."` (or the `deserialize` half of `rename(...)`).
    pub rename: Option<String>,
    /// `default` or `default = "path"`: a missing key is not an error.
    pub default: bool,
    /// `with = "module"`: both directions use a codec module.
    pub with: Option<String>,
    /// `deserialize_with = "path"`: decoding uses a custom function.
    pub deserialize_with: Option<String>,
    /// `flatten`: the field's keys live on the parent object.
    pub flatten: bool,
    /// `skip` or `skip_deserializing`: the field is never read from JSON.
    pub skip: bool,
    /// `skip_serializing_if = "path"`: the predicate that omits the member
    /// when serializing (`Option::is_none` is the only one gen_go knows).
    pub skip_serializing_if: Option<String>,
}

/// The composition wrappers peeled from a field type, outermost first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wrap {
    Option,
    Vec,
}

impl FieldDef {
    /// The JSON key this field is read from: `rename` if set, else the
    /// container's `rename_all` rule applied to the Rust name, else the
    /// Rust name itself (with any `r#` prefix removed).
    pub fn wire_name(&self, rename_all: Option<&str>) -> String {
        if let Some(rename) = &self.attrs.rename {
            return rename.clone();
        }
        let base = self.name.strip_prefix("r#").unwrap_or(&self.name);
        match rename_all {
            Some(rule) => apply_rename_rule(rule, base),
            None => base.to_string(),
        }
    }

    /// Whether the JSON key must be present: the type is not `Option<_>`
    /// and the field carries no `default`.
    pub fn required(&self) -> bool {
        !self.attrs.default && !is_option(&self.ty)
    }
}

fn is_option(ty: &Type) -> bool {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "Option"),
        _ => false,
    }
}

/// Applies a serde `rename_all` rule to a snake_case Rust field name. An
/// unknown rule leaves the name unchanged.
pub fn apply_rename_rule(rule: &str, name: &str) -> String {
    match rule {
        "lowercase" => name.to_ascii_lowercase(),
        "UPPERCASE" => name.to_ascii_uppercase(),
        "PascalCase" => pascal_case(name),
        "camelCase" => {
            let pascal = pascal_case(name);
            let mut chars = pascal.chars();
            match chars.next() {
                Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
                None => pascal,
            }
        }
        "snake_case" => name.to_string(),
        "SCREAMING_SNAKE_CASE" => name.to_ascii_uppercase(),
        "kebab-case" => name.replace('_', "-"),
        "SCREAMING-KEBAB-CASE" => name.to_ascii_uppercase().replace('_', "-"),
        _ => name.to_string(),
    }
}

fn pascal_case(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut capitalize = true;
    for ch in name.chars() {
        if ch == '_' {
            capitalize = true;
        } else if capitalize {
            out.extend(ch.to_uppercase());
            capitalize = false;
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(name: &str, ty: &str, attrs: FieldAttrs) -> FieldDef {
        FieldDef {
            name: name.to_string(),
            ty: syn::parse_str(ty).expect("valid type"),
            attrs,
        }
    }

    #[test]
    fn rename_rules_match_serde() {
        assert_eq!(apply_rename_rule("camelCase", "market_cap"), "marketCap");
        assert_eq!(apply_rename_rule("camelCase", "symbol"), "symbol");
        assert_eq!(apply_rename_rule("PascalCase", "market_cap"), "MarketCap");
        assert_eq!(
            apply_rename_rule("SCREAMING_SNAKE_CASE", "market_cap"),
            "MARKET_CAP"
        );
        assert_eq!(apply_rename_rule("kebab-case", "market_cap"), "market-cap");
        assert_eq!(
            apply_rename_rule("SCREAMING-KEBAB-CASE", "market_cap"),
            "MARKET-CAP"
        );
        assert_eq!(apply_rename_rule("lowercase", "Market_cap"), "market_cap");
        assert_eq!(apply_rename_rule("UPPERCASE", "market_cap"), "MARKET_CAP");
        assert_eq!(apply_rename_rule("snake_case", "market_cap"), "market_cap");
        assert_eq!(apply_rename_rule("bogus", "market_cap"), "market_cap");
    }

    #[test]
    fn wire_name_prefers_rename_then_rule_then_rust_name() {
        let renamed = field(
            "one_day",
            "f64",
            FieldAttrs {
                rename: Some("1D".to_string()),
                ..FieldAttrs::default()
            },
        );
        assert_eq!(renamed.wire_name(Some("camelCase")), "1D");
        let plain = field("market_cap", "f64", FieldAttrs::default());
        assert_eq!(plain.wire_name(Some("camelCase")), "marketCap");
        assert_eq!(plain.wire_name(None), "market_cap");
        let raw = field("r#type", "String", FieldAttrs::default());
        assert_eq!(raw.wire_name(Some("camelCase")), "type");
    }

    #[test]
    fn required_needs_non_option_without_default() {
        assert!(field("price", "Price", FieldAttrs::default()).required());
        assert!(!field("price", "Option<Price>", FieldAttrs::default()).required());
        assert!(field("bars", "Vec<Option<Price>>", FieldAttrs::default()).required());
        let defaulted = FieldAttrs {
            default: true,
            ..FieldAttrs::default()
        };
        assert!(!field("price", "Price", defaulted).required());
    }
}
