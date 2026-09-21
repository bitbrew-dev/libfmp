//! The language-neutral intermediate representation of a discovered response
//! struct: its name, module path, serde container rule, and named fields with
//! their `syn` types and serde field attributes.

use syn::Type;

/// A single response struct discovered in libfmp.
#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub module_path: Vec<String>,
    /// The container-level `#[serde(rename_all = "...")]` rule, if any.
    pub rename_all: Option<String>,
    pub fields: Vec<FieldDef>,
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
