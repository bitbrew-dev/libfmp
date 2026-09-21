#![allow(dead_code)]

//! The raw facts the wire resolver evaluates, captured per parsed file:
//! free functions (descriptor functions and their helpers), the descriptor
//! call inside every `Client` method, every `QueryParameters::encode` body,
//! struct field types (for `self.field.encode(encoder)` delegation, where
//! the target's `encode` may be an inherent method, as `DcfAssumptions`), the
//! `const fn (self) -> &'static str` match tables enums use to pick a path,
//! and `&[&str]` constants (the binary content-type lists).
//!
//! Macro-generated items are captured through the same
//! [`visit_items`](super::super::scan::visit_items) walk the validator uses,
//! so descriptor functions emitted by a file-level `macro_rules!` (the
//! congressional `paginated_endpoint!` family) look like direct ones.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use quote::ToTokens;
use syn::{Expr, Fields, ImplItem, Item, Lit, Pat, ReturnType, Stmt, Type};

use super::super::scan::{
    Origin, Unexpanded, base_ident, is_public, module_path, typed_params, visit_items,
};

/// A `libfmp::endpoints` module path, as [`module_path`] produces it.
pub(super) type ModulePath = Vec<String>;

/// One free function under `endpoints/`.
#[derive(Debug, Clone)]
pub(super) struct FnDef {
    pub file: PathBuf,
    pub module: ModulePath,
    pub origin: Origin,
    pub params: Vec<String>,
    pub output: ReturnType,
    pub body: syn::Block,
}

/// One `pub async fn` on `impl Client`.
#[derive(Debug, Clone)]
pub(super) struct ClientMethodDef {
    pub module: ModulePath,
    pub body: syn::Block,
}

/// Everything the resolver reads, keyed for lookup.
#[derive(Debug, Default)]
pub(super) struct Collected {
    pub functions: BTreeMap<(ModulePath, String), FnDef>,
    pub client_methods: BTreeMap<String, ClientMethodDef>,
    /// `QueryParameters::encode` bodies keyed by the implementing type.
    pub encoders: BTreeMap<String, syn::Block>,
    /// Named struct fields keyed by struct, then field, to the base type.
    pub struct_fields: BTreeMap<String, BTreeMap<String, String>>,
    /// `(type, method)` to `{variant: literal}` for
    /// `fn method(self) -> &'static str { match self { Self::V => "..", } }`.
    pub literal_methods: BTreeMap<(String, String), BTreeMap<String, String>>,
    /// `const NAME: &[&str] = &["..", ..];` values keyed by name.
    pub string_lists: BTreeMap<String, Vec<String>>,
    pub unexpanded: Vec<Unexpanded>,
}

impl Collected {
    pub(super) fn from_files(endpoints_root: &Path, files: &[(PathBuf, syn::File)]) -> Self {
        let mut collected = Collected::default();
        for (file, parsed) in files {
            let module = module_path(endpoints_root, file);
            let mut unexpanded = Vec::new();
            visit_items(file, parsed, &mut unexpanded, |item, origin, _| {
                collected.absorb(file, &module, item, origin);
            });
            collected.unexpanded.extend(unexpanded);
        }
        collected
    }

    fn absorb(&mut self, file: &Path, module: &ModulePath, item: &Item, origin: &Origin) {
        match item {
            Item::Fn(item) => {
                self.functions.insert(
                    (module.clone(), item.sig.ident.to_string()),
                    FnDef {
                        file: file.to_path_buf(),
                        module: module.clone(),
                        origin: origin.clone(),
                        params: typed_params(&item.sig)
                            .into_iter()
                            .map(|param| param.name)
                            .collect(),
                        output: item.sig.output.clone(),
                        body: (*item.block).clone(),
                    },
                );
            }
            Item::Struct(item) => {
                let fields = match &item.fields {
                    Fields::Named(named) => named
                        .named
                        .iter()
                        .filter_map(|field| {
                            let name = field.ident.as_ref()?.to_string();
                            Some((name, base_ident(&field.ty).unwrap_or_default()))
                        })
                        .collect(),
                    _ => BTreeMap::new(),
                };
                self.struct_fields.insert(item.ident.to_string(), fields);
            }
            Item::Const(item) => {
                if let Some(list) = string_list(&item.expr) {
                    self.string_lists.insert(item.ident.to_string(), list);
                }
            }
            Item::Impl(item) => self.absorb_impl(module, item),
            _ => {}
        }
    }

    fn absorb_impl(&mut self, module: &ModulePath, item: &syn::ItemImpl) {
        let Some(self_ty) = base_ident(&item.self_ty) else {
            return;
        };
        if let Some((_, trait_path, _)) = &item.trait_ {
            if trait_path
                .segments
                .last()
                .is_some_and(|s| s.ident == "QueryParameters")
                && let Some(encode) = item.items.iter().find_map(|method| match method {
                    ImplItem::Fn(method) if method.sig.ident == "encode" => Some(method),
                    _ => None,
                })
            {
                self.encoders.insert(self_ty, encode.block.clone());
            }
            return;
        }
        for method in &item.items {
            let ImplItem::Fn(method) = method else {
                continue;
            };
            if self_ty == "Client" {
                if is_public(&method.vis) && method.sig.asyncness.is_some() {
                    self.client_methods.insert(
                        method.sig.ident.to_string(),
                        ClientMethodDef {
                            module: module.clone(),
                            body: method.block.clone(),
                        },
                    );
                }
            } else if method.sig.ident == "encode" {
                self.encoders
                    .entry(self_ty.clone())
                    .or_insert_with(|| method.block.clone());
            } else if let Some(table) = literal_match_table(&method.block) {
                self.literal_methods
                    .insert((self_ty.clone(), method.sig.ident.to_string()), table);
            }
        }
    }
}

/// Reads `&["a", "b"]` (or `["a", "b"]`) into its string values.
fn string_list(expr: &Expr) -> Option<Vec<String>> {
    match expr {
        Expr::Reference(reference) => string_list(&reference.expr),
        Expr::Array(array) => array.elems.iter().map(string_literal).collect(),
        _ => None,
    }
}

/// The value of a string literal expression.
pub(super) fn string_literal(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(literal) => match &literal.lit {
            Lit::Str(value) => Some(value.value()),
            _ => None,
        },
        _ => None,
    }
}

/// Reads a body that is exactly `match self { Type::V => "lit", .. }` into
/// its `{variant: literal}` table; any other shape gives `None`.
fn literal_match_table(block: &syn::Block) -> Option<BTreeMap<String, String>> {
    let [Stmt::Expr(Expr::Match(matched), None)] = block.stmts.as_slice() else {
        return None;
    };
    if !matches!(&*matched.expr, Expr::Path(path) if path.path.is_ident("self")) {
        return None;
    }
    matched
        .arms
        .iter()
        .map(|arm| {
            let Pat::Path(pattern) = &arm.pat else {
                return None;
            };
            let variant = pattern.path.segments.last()?.ident.to_string();
            Some((variant, string_literal(&arm.body)?))
        })
        .collect()
}

/// The first generic argument of an `EndpointSpec<Q, R>` return type: the
/// query type name, or `None` when `Q` is the unit type.
pub(super) fn query_type(output: &ReturnType) -> Result<Option<String>, String> {
    let ReturnType::Type(_, ty) = output else {
        return Err("descriptor has no return type".to_owned());
    };
    let Type::Path(path) = &**ty else {
        return Err("descriptor does not return `EndpointSpec<Q, R>`".to_owned());
    };
    let segment = path
        .path
        .segments
        .last()
        .filter(|segment| segment.ident == "EndpointSpec")
        .ok_or_else(|| "descriptor does not return `EndpointSpec<Q, R>`".to_owned())?;
    let syn::PathArguments::AngleBracketed(args) = &segment.arguments else {
        return Err("`EndpointSpec` return type has no generic arguments".to_owned());
    };
    match args.args.first() {
        Some(syn::GenericArgument::Type(Type::Tuple(tuple))) if tuple.elems.is_empty() => Ok(None),
        Some(syn::GenericArgument::Type(query)) => base_ident(query)
            .map(Some)
            .ok_or_else(|| "query type of `EndpointSpec<Q, R>` is not a path".to_owned()),
        _ => Err("`EndpointSpec` return type has no query argument".to_owned()),
    }
}

/// The source text of a syntax node, for error messages.
pub(super) fn text(tokens: &impl ToTokens) -> String {
    tokens.to_token_stream().to_string()
}
