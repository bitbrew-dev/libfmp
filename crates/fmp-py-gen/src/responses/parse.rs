//! Source discovery and `syn` parsing: walking response files, deriving module
//! paths, peeling wrapper types, and reading serde attributes.

use std::fs;
use std::path::{Path, PathBuf};

use syn::meta::ParseNestedMeta;
use syn::{Attribute, Expr, GenericArgument, Item, LitStr, PathArguments, Token, Type};

use super::DiscoverError;
use super::model::{FieldAttrs, FieldDef, StructDef, Wrap};

/// Peels `Option`/`Vec` wrappers and returns the base type identifier.
pub fn peel(ty: &Type) -> (Vec<Wrap>, Option<String>) {
    let mut wraps = Vec::new();
    let mut current = ty;
    loop {
        let Type::Path(path) = current else {
            return (wraps, None);
        };
        let Some(segment) = path.path.segments.last() else {
            return (wraps, None);
        };
        let ident = segment.ident.to_string();
        if (ident == "Option" || ident == "Vec")
            && let PathArguments::AngleBracketed(args) = &segment.arguments
            && let Some(GenericArgument::Type(inner)) = args.args.first()
        {
            wraps.push(if ident == "Option" {
                Wrap::Option
            } else {
                Wrap::Vec
            });
            current = inner;
            continue;
        }
        return (wraps, Some(ident));
    }
}

/// Returns the last path-segment identifier of a type, ignoring generics.
pub fn base_ident(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(path) => path.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    }
}

/// Reports whether an item is `pub`.
pub fn is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

/// Reconstructs the technical-indicator structs hidden behind a declarative macro.
pub fn technical_indicator_structs(
    file: &syn::File,
    module_path: &[String],
) -> Result<Vec<StructDef>, DiscoverError> {
    let mut defs = Vec::new();
    for item in &file.items {
        let Item::Macro(item) = item else { continue };
        if item.mac.path.segments.last().map(|s| s.ident.to_string())
            != Some("technical_indicator_row".to_string())
        {
            continue;
        }
        let tokens = item.mac.tokens.to_string();
        let parts: Vec<String> = tokens
            .split(',')
            .map(|part| part.trim().to_string())
            .collect();
        if parts.len() != 3 {
            return Err(DiscoverError::MacroArgs(tokens));
        }
        let name = parts[0].clone();
        let metric = parts[1].clone();
        let metric_type = parts[2].clone();
        let mut fields = Vec::new();
        for (field_name, field_ty) in [
            ("date", "ApiDateTime"),
            ("open", "Price"),
            ("high", "Price"),
            ("low", "Price"),
            ("close", "Price"),
            ("volume", "Volume"),
        ] {
            fields.push(FieldDef {
                name: field_name.to_string(),
                ty: parse_type(field_ty)?,
                attrs: FieldAttrs::default(),
            });
        }
        fields.push(FieldDef {
            name: metric,
            ty: parse_type(&metric_type)?,
            attrs: FieldAttrs::default(),
        });
        defs.push(StructDef {
            name,
            module_path: module_path.to_vec(),
            doc: None,
            rename_all: Some("camelCase".to_string()),
            fields,
            custom_deserialize: false,
        });
    }
    Ok(defs)
}

fn parse_type(text: &str) -> Result<Type, DiscoverError> {
    syn::parse_str::<Type>(text).map_err(|source| DiscoverError::MacroType {
        text: text.to_string(),
        source,
    })
}

/// Reads the first paragraph of an item's `///` doc comment: consecutive
/// non-blank `#[doc = "..."]` lines, trimmed and joined with one space.
/// Returns `None` when the item has no doc comment.
pub fn doc_paragraph(attrs: &[Attribute]) -> Option<String> {
    let mut lines = Vec::new();
    for attr in attrs {
        if !attr.path().is_ident("doc") {
            continue;
        }
        let syn::Meta::NameValue(meta) = &attr.meta else {
            continue;
        };
        let Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(text),
            ..
        }) = &meta.value
        else {
            continue;
        };
        let line = text.value().trim().to_string();
        if line.is_empty() {
            if !lines.is_empty() {
                break;
            }
            continue;
        }
        lines.push(line);
    }
    if lines.is_empty() {
        None
    } else {
        Some(lines.join(" "))
    }
}

/// Reads the container-level `rename_all` rule from a struct's attributes.
pub fn struct_rename_all(attrs: &[Attribute]) -> syn::Result<Option<String>> {
    let mut rule = None;
    for attr in serde_attrs(attrs) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename_all") {
                rule = string_value(&meta)?;
            } else {
                skip_payload(&meta)?;
            }
            Ok(())
        })?;
    }
    Ok(rule)
}

/// Reads the serde facts a field carries in its `#[serde(...)]` attributes.
pub fn field_attrs(attrs: &[Attribute]) -> syn::Result<FieldAttrs> {
    let mut out = FieldAttrs::default();
    for attr in serde_attrs(attrs) {
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("rename") {
                out.rename = string_value(&meta)?;
            } else if meta.path.is_ident("default") {
                out.default = true;
                skip_payload(&meta)?;
            } else if meta.path.is_ident("with") {
                out.with = string_value(&meta)?;
            } else if meta.path.is_ident("deserialize_with") {
                out.deserialize_with = string_value(&meta)?;
            } else if meta.path.is_ident("serialize_with") {
                out.serialize_with = string_value(&meta)?;
            } else if meta.path.is_ident("flatten") {
                out.flatten = true;
            } else if meta.path.is_ident("skip") || meta.path.is_ident("skip_deserializing") {
                out.skip = true;
            } else if meta.path.is_ident("skip_serializing_if") {
                out.skip_serializing_if = string_value(&meta)?;
            } else {
                skip_payload(&meta)?;
            }
            Ok(())
        })?;
    }
    Ok(out)
}

fn serde_attrs(attrs: &[Attribute]) -> impl Iterator<Item = &Attribute> {
    attrs.iter().filter(|attr| attr.path().is_ident("serde"))
}

/// Reads `key = "value"`, or the `deserialize` half of
/// `key(serialize = "...", deserialize = "...")`.
fn string_value(meta: &ParseNestedMeta) -> syn::Result<Option<String>> {
    if meta.input.peek(Token![=]) {
        let lit: LitStr = meta.value()?.parse()?;
        return Ok(Some(lit.value()));
    }
    let mut found = None;
    meta.parse_nested_meta(|inner| {
        let lit: LitStr = inner.value()?.parse()?;
        if inner.path.is_ident("deserialize") {
            found = Some(lit.value());
        }
        Ok(())
    })?;
    Ok(found)
}

/// Consumes the payload of an attribute key this reader does not use, so the
/// nested-meta parser can continue to the next key.
fn skip_payload(meta: &ParseNestedMeta) -> syn::Result<()> {
    if meta.input.peek(Token![=]) {
        meta.value()?.parse::<Expr>()?;
    } else if meta.input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in meta.input);
        content.parse::<proc_macro2::TokenStream>()?;
    }
    Ok(())
}

/// Derives a module path from a response file's location.
pub fn module_path_for(file: &Path, root: &Path) -> Vec<String> {
    let rel = file.strip_prefix(root).unwrap_or(file);
    let mut comps: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    if let Some(last) = comps.pop() {
        let stem = last.strip_suffix(".rs").unwrap_or(&last).to_string();
        if stem != "mod" {
            comps.push(stem);
        }
    }
    comps
}

/// Recursively collects every `.rs` file under a directory.
pub fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), DiscoverError> {
    let entries = fs::read_dir(dir).map_err(|source| DiscoverError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let path = entry
            .map_err(|source| DiscoverError::Io {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_struct(source: &str) -> syn::ItemStruct {
        syn::parse_str(source).expect("valid struct")
    }

    fn attrs_of(item: &syn::ItemStruct, index: usize) -> FieldAttrs {
        let field = item.fields.iter().nth(index).expect("field exists");
        field_attrs(&field.attrs).expect("attributes parse")
    }

    #[test]
    fn captures_rename_default_with_deserialize_with_and_serialize_with() {
        let item = parse_struct(
            r#"
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct Row {
                #[serde(rename = "1D")]
                pub one_day: f64,
                #[serde(default)]
                pub notes: Vec<String>,
                #[serde(with = "empty_date")]
                pub filed: Option<Date>,
                #[serde(deserialize_with = "required_option")]
                pub price: Option<Price>,
                #[serde(flatten, skip_deserializing, bound = "T: Clone")]
                pub extra: Extra,
                #[serde(rename(serialize = "out", deserialize = "in"), default = "zero")]
                pub both: u32,
                #[serde(rename = "flagUSD", skip_serializing_if = "Option::is_none")]
                pub flag: Option<Flag>,
                #[doc = "no serde"]
                pub plain: String,
                #[serde(serialize_with = "crate::codecs::volume::serialize")]
                pub volume: Volume,
            }
            "#,
        );
        assert_eq!(
            struct_rename_all(&item.attrs).expect("rename_all parses"),
            Some("camelCase".to_string())
        );
        let one_day = attrs_of(&item, 0);
        assert_eq!(one_day.rename.as_deref(), Some("1D"));
        assert!(!one_day.default);
        assert!(attrs_of(&item, 1).default);
        assert_eq!(attrs_of(&item, 2).with.as_deref(), Some("empty_date"));
        assert_eq!(
            attrs_of(&item, 3).deserialize_with.as_deref(),
            Some("required_option")
        );
        let extra = attrs_of(&item, 4);
        assert!(extra.flatten);
        assert!(extra.skip);
        let both = attrs_of(&item, 5);
        assert_eq!(both.rename.as_deref(), Some("in"));
        assert!(both.default);
        let flag = attrs_of(&item, 6);
        assert_eq!(flag.rename.as_deref(), Some("flagUSD"));
        assert_eq!(flag.skip_serializing_if.as_deref(), Some("Option::is_none"));
        assert_eq!(attrs_of(&item, 7), FieldAttrs::default());
        let volume = attrs_of(&item, 8);
        assert_eq!(
            volume.serialize_with.as_deref(),
            Some("crate::codecs::volume::serialize")
        );
        assert_eq!(
            FieldAttrs {
                serialize_with: None,
                ..volume
            },
            FieldAttrs::default(),
            "serialize_with must not set any attribute a generator acts on"
        );
    }

    #[test]
    fn doc_paragraph_joins_the_first_paragraph_only() {
        let item = parse_struct(
            r#"
            /// A detailed
            ///   real-time quote.
            ///
            /// Second paragraph is ignored.
            #[derive(Deserialize)]
            pub struct Row { pub price: f64 }
            "#,
        );
        assert_eq!(
            doc_paragraph(&item.attrs).as_deref(),
            Some("A detailed real-time quote.")
        );
        let bare = parse_struct("pub struct Row { pub price: f64 }");
        assert_eq!(doc_paragraph(&bare.attrs), None);
    }

    #[test]
    fn peel_strips_option_and_vec_outermost_first() {
        let ty: Type = syn::parse_str("Option<Vec<Price>>").expect("valid type");
        assert_eq!(
            peel(&ty),
            (vec![Wrap::Option, Wrap::Vec], Some("Price".into()))
        );
        let ty: Type = syn::parse_str("Option<Option<f64>>").expect("valid type");
        assert_eq!(
            peel(&ty),
            (vec![Wrap::Option, Wrap::Option], Some("f64".into()))
        );
        let ty: Type = syn::parse_str("libfmp::types::Ticker").expect("valid type");
        assert_eq!(peel(&ty), (Vec::new(), Some("Ticker".into())));
        let ty: Type = syn::parse_str("(u8, u8)").expect("valid type");
        assert_eq!(peel(&ty), (Vec::new(), None));
    }
}
