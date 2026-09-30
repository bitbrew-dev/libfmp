//! The `libfmp` surface the registry is checked against: every
//! `pub async fn` on `impl Client` and every query type's constructor and
//! `with_*` setters, read with `syn` from `crates/libfmp/src/endpoints/**`.
//!
//! Query types emitted by `macro_rules!` are recovered through
//! [`expand`](super::expand), and so are setters emitted by a macro invoked
//! inside a query type's `impl` block; invocations the expander cannot
//! handle are listed in [`Surface::unexpanded`] so the validator can report
//! the affected query types as trusted rather than verified.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use proc_macro2::TokenStream;
use syn::{FnArg, GenericArgument, ImplItem, Item, PathArguments, ReturnType, Type, Visibility};

use super::expand;

/// How a query type's definition was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Origin {
    /// Written out directly in the source file.
    Direct,
    /// Recovered by expanding the named `macro_rules!` invocation.
    Macro(String),
}

/// One parameter of a constructor or setter, reduced to the base type ident
/// after peeling `impl Into<T>`, `Option<T>`, and references.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Param {
    pub name: String,
    pub base_type: String,
}

/// The constructor and setters of one query type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryApi {
    pub file: PathBuf,
    pub origin: Origin,
    /// Parameters of `pub fn new`, or `None` when no public `new` exists.
    pub ctor: Option<Vec<Param>>,
    /// Public `with_*` builder methods keyed by name.
    pub setters: BTreeMap<String, Param>,
}

/// What a client method returns.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Returns {
    /// `Result<Vec<R>>`, carrying the row type name.
    Rows(String),
    /// `Result<R>` for one typed object, carrying its type name.
    Single(String),
    /// `Result<BinaryResponse>`.
    Binary,
    /// Anything else, carried verbatim for the error message.
    Other(String),
}

/// One `pub async fn` on `impl Client`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientMethod {
    pub file: PathBuf,
    /// The query type name, whether taken as `Q` or `impl Into<Q>`.
    pub query: Option<String>,
    pub returns: Returns,
}

/// A macro invocation the expander could not turn into items.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unexpanded {
    pub file: PathBuf,
    pub macro_name: String,
    pub reason: String,
}

/// Everything the validator needs from `libfmp`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Surface {
    pub methods: BTreeMap<String, ClientMethod>,
    pub queries: BTreeMap<String, QueryApi>,
    pub unexpanded: Vec<Unexpanded>,
}

/// The `libfmp::endpoints` module path of a source file, as segments:
/// `statements.rs` and `statements/mod.rs` both give `["statements"]`,
/// `statements/reports.rs` gives `["statements", "reports"]`, and the root
/// `mod.rs` gives an empty path.
pub fn module_path(endpoints_root: &Path, file: &Path) -> Vec<String> {
    let relative = file.strip_prefix(endpoints_root).unwrap_or(file);
    let mut segments: Vec<String> = relative
        .with_extension("")
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    if segments.last().is_some_and(|last| last == "mod") {
        segments.pop();
    }
    segments
}

/// Why the endpoint tree could not be read.
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: {source}")]
    Syntax {
        path: PathBuf,
        #[source]
        source: syn::Error,
    },
}

impl Surface {
    /// Parses every `.rs` file under `endpoints_root`.
    pub fn scan(endpoints_root: &Path) -> Result<Surface, ScanError> {
        let mut files = Vec::new();
        collect_rust_files(endpoints_root, &mut files)?;
        files.sort();
        let mut surface = Surface::default();
        for file in &files {
            let content = fs::read_to_string(file).map_err(|source| ScanError::Io {
                path: file.clone(),
                source,
            })?;
            let parsed = syn::parse_file(&content).map_err(|source| ScanError::Syntax {
                path: file.clone(),
                source,
            })?;
            surface.absorb_file(file, &parsed);
        }
        Ok(surface)
    }

    fn absorb_file(&mut self, file: &Path, parsed: &syn::File) {
        let mut unexpanded = Vec::new();
        visit_items(
            file,
            parsed,
            &mut unexpanded,
            |item, origin, definitions| {
                self.absorb_item(file, item, origin, definitions);
            },
        );
        self.unexpanded.extend(unexpanded);
    }

    fn absorb_item(
        &mut self,
        file: &Path,
        item: &Item,
        origin: &Origin,
        definitions: &Definitions,
    ) {
        let Item::Impl(item) = item else { return };
        if item.trait_.is_some() {
            return;
        }
        let Some(self_ty) = base_ident(&item.self_ty) else {
            return;
        };
        if self_ty == "Client" {
            for method in &item.items {
                if let ImplItem::Fn(method) = method
                    && is_public(&method.vis)
                    && method.sig.asyncness.is_some()
                {
                    self.methods.insert(
                        method.sig.ident.to_string(),
                        client_method(file, &method.sig),
                    );
                }
            }
            return;
        }
        let api = self.queries.entry(self_ty).or_insert_with(|| QueryApi {
            file: file.to_path_buf(),
            origin: origin.clone(),
            ctor: None,
            setters: BTreeMap::new(),
        });
        let mut unexpanded = Vec::new();
        for method in &item.items {
            match method {
                ImplItem::Fn(method) => absorb_query_method(api, method),
                ImplItem::Macro(invocation) => {
                    let Some(name) = invocation.mac.path.get_ident().map(ToString::to_string)
                    else {
                        continue;
                    };
                    let Some(body) = definitions.get(&name) else {
                        continue;
                    };
                    match expand::expand_impl_items(body, &invocation.mac.tokens) {
                        Ok(expanded) => {
                            for method in &expanded {
                                if let ImplItem::Fn(method) = method {
                                    absorb_query_method(api, method);
                                }
                            }
                        }
                        Err(reason) => unexpanded.push(Unexpanded {
                            file: file.to_path_buf(),
                            macro_name: name,
                            reason: reason.to_string(),
                        }),
                    }
                }
                _ => {}
            }
        }
        self.unexpanded.extend(unexpanded);
    }
}

/// The `macro_rules!` definitions of one file, keyed by macro name.
pub(crate) type Definitions = BTreeMap<String, TokenStream>;

/// Visits every item of a parsed file: the direct items first, then the
/// items each file-level macro invocation expands to. Invocations the
/// expander cannot handle are appended to `unexpanded`; definitions are
/// passed along so visitors can expand `impl`-level invocations.
pub(crate) fn visit_items(
    file: &Path,
    parsed: &syn::File,
    unexpanded: &mut Vec<Unexpanded>,
    mut visit: impl FnMut(&Item, &Origin, &Definitions),
) {
    let definitions: Definitions = parsed
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Macro(item) => expand::definition(item),
            _ => None,
        })
        .collect();
    let mut invocations = Vec::new();
    for item in &parsed.items {
        match item {
            Item::Macro(item) if expand::definition(item).is_none() => {
                invocations.push(item);
            }
            Item::Macro(_) => {}
            other => visit(other, &Origin::Direct, &definitions),
        }
    }
    for invocation in invocations {
        let Some(name) = invocation.mac.path.get_ident().map(ToString::to_string) else {
            continue;
        };
        let Some(body) = definitions.get(&name) else {
            continue;
        };
        match expand::expand(body, &invocation.mac.tokens) {
            Ok(expanded) => {
                let origin = Origin::Macro(name);
                for item in &expanded.items {
                    visit(item, &origin, &definitions);
                }
            }
            Err(reason) => unexpanded.push(Unexpanded {
                file: file.to_path_buf(),
                macro_name: name,
                reason: reason.to_string(),
            }),
        }
    }
}

/// Records a public `new` constructor or single-parameter `with_*` setter.
fn absorb_query_method(api: &mut QueryApi, method: &syn::ImplItemFn) {
    if !is_public(&method.vis) {
        return;
    }
    let name = method.sig.ident.to_string();
    let params = typed_params(&method.sig);
    if name == "new" {
        api.ctor = Some(params);
    } else if name.starts_with("with_")
        && let [param] = params.as_slice()
    {
        api.setters.insert(name, param.clone());
    }
}

fn client_method(file: &Path, sig: &syn::Signature) -> ClientMethod {
    let query = typed_params(sig).into_iter().next().map(|p| p.base_type);
    let returns = match &sig.output {
        ReturnType::Type(_, ty) => classify_return(ty),
        ReturnType::Default => Returns::Other("()".to_owned()),
    };
    ClientMethod {
        file: file.to_path_buf(),
        query,
        returns,
    }
}

/// Reads `Result<Vec<R>>`, `Result<BinaryResponse>`, and `Result<R>`.
fn classify_return(ty: &Type) -> Returns {
    let verbatim = || Returns::Other(quote_type(ty));
    let Some((head, inner)) = generic_head(ty) else {
        return verbatim();
    };
    if head != "Result" {
        return verbatim();
    }
    match generic_head(inner) {
        Some((head, row)) if head == "Vec" => base_ident(row).map_or_else(verbatim, Returns::Rows),
        None => match base_ident(inner) {
            Some(name) if name == "BinaryResponse" => Returns::Binary,
            Some(name) => Returns::Single(name),
            None => verbatim(),
        },
        _ => verbatim(),
    }
}

/// The named (non-`self`) parameters of a signature.
pub(crate) fn typed_params(sig: &syn::Signature) -> Vec<Param> {
    sig.inputs
        .iter()
        .filter_map(|input| match input {
            FnArg::Typed(pat) => Some(Param {
                name: quote_pat(&pat.pat),
                base_type: peel_base(&pat.ty),
            }),
            FnArg::Receiver(_) => None,
        })
        .collect()
}

/// Peels `impl Into<T>`, `Option<T>`, and `&T` down to the base type ident.
fn peel_base(ty: &Type) -> String {
    match ty {
        Type::Reference(reference) => peel_base(&reference.elem),
        Type::ImplTrait(impl_trait) => impl_trait
            .bounds
            .iter()
            .find_map(|bound| match bound {
                syn::TypeParamBound::Trait(bound) => bound.path.segments.last(),
                _ => None,
            })
            .and_then(|segment| match &segment.arguments {
                PathArguments::AngleBracketed(args) if segment.ident == "Into" => {
                    args.args.iter().find_map(|arg| match arg {
                        GenericArgument::Type(inner) => Some(peel_base(inner)),
                        _ => None,
                    })
                }
                _ => None,
            })
            .unwrap_or_else(|| quote_type(ty)),
        _ => match generic_head(ty) {
            Some((head, inner)) if head == "Option" => peel_base(inner),
            _ => base_ident(ty).unwrap_or_else(|| quote_type(ty)),
        },
    }
}

/// Splits `Head<Inner>` into its head ident and first type argument.
pub(crate) fn generic_head(ty: &Type) -> Option<(String, &Type)> {
    let Type::Path(path) = ty else { return None };
    let segment = path.path.segments.last()?;
    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return None;
    };
    let inner = args.args.iter().find_map(|arg| match arg {
        GenericArgument::Type(inner) => Some(inner),
        _ => None,
    })?;
    Some((segment.ident.to_string(), inner))
}

/// The last path-segment identifier of a type, ignoring generics.
pub(crate) fn base_ident(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(path) => path.path.segments.last().map(|s| s.ident.to_string()),
        _ => None,
    }
}

fn quote_type(ty: &Type) -> String {
    quote_tokens(ty)
}

fn quote_pat(pat: &syn::Pat) -> String {
    match pat {
        syn::Pat::Ident(ident) => ident.ident.to_string(),
        other => quote_tokens(other),
    }
}

fn quote_tokens(tokens: &impl quote::ToTokens) -> String {
    tokens.to_token_stream().to_string().replace(' ', "")
}

pub(crate) fn is_public(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

/// Recursively collects every `.rs` file under a directory.
pub(crate) fn collect_rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), ScanError> {
    let entries = fs::read_dir(dir).map_err(|source| ScanError::Io {
        path: dir.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let path = entry
            .map_err(|source| ScanError::Io {
                path: dir.to_path_buf(),
                source,
            })?
            .path();
        if path.is_dir() {
            collect_rust_files(&path, out)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surface() -> Surface {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../libfmp/src/endpoints");
        Surface::scan(&root).expect("endpoints parse")
    }

    #[test]
    fn finds_every_client_method() {
        let surface = surface();
        assert_eq!(surface.methods.len(), 271);
        let quote = &surface.methods["quote"];
        assert_eq!(quote.query.as_deref(), Some("QuoteQuery"));
        assert_eq!(quote.returns, Returns::Rows("Quote".to_owned()));
        assert_eq!(surface.methods["mutual_fund_quotes"].query, None);
        assert_eq!(
            surface.methods["financial_reports_xlsx"].returns,
            Returns::Binary
        );
        let singles: Vec<&str> = surface
            .methods
            .iter()
            .filter(|(_, method)| matches!(method.returns, Returns::Single(_)))
            .map(|(name, _)| name.as_str())
            .collect();
        assert_eq!(singles, ["financial_reports_json"]);
    }

    #[test]
    fn recovers_macro_emitted_queries() {
        let surface = surface();
        let quote = &surface.queries["QuoteQuery"];
        assert_eq!(quote.origin, Origin::Macro("symbol_query".to_owned()));
        assert_eq!(
            quote.ctor.as_deref(),
            Some(
                &[Param {
                    name: "symbol".to_owned(),
                    base_type: "Ticker".to_owned()
                }][..]
            )
        );
        let income = &surface.queries["IncomeStatementQuery"];
        assert_eq!(income.setters["with_limit"].base_type, "Limit");
        assert_eq!(income.setters["with_period"].base_type, "StatementPeriod");
    }

    #[test]
    fn recovers_setters_emitted_by_impl_level_macros() {
        let surface = surface();
        let screener = &surface.queries["CompanyScreenerQuery"];
        assert_eq!(screener.origin, Origin::Direct);
        assert_eq!(screener.ctor.as_deref(), Some(&[][..]));
        assert_eq!(screener.setters.len(), 20);
        assert_eq!(
            screener.setters["with_market_cap_more_than"].base_type,
            "u64"
        );
        assert_eq!(screener.setters["with_sector"].base_type, "Sector");
        assert_eq!(screener.setters["with_is_etf"].base_type, "bool");
        assert_eq!(screener.setters["with_limit"].base_type, "Limit");
        let dcf = &surface.queries["DcfAssumptions"];
        assert_eq!(dcf.setters["with_beta"].base_type, "FiniteDecimal");
        assert!(
            surface
                .unexpanded
                .iter()
                .all(|e| !["value_filter", "string_filter", "assumption"]
                    .contains(&e.macro_name.as_str())),
            "setter macros must expand: {:?}",
            surface.unexpanded
        );
    }

    #[test]
    fn unexpanded_macros_are_listed_not_fatal() {
        let surface = surface();
        for entry in &surface.unexpanded {
            eprintln!(
                "{}: {}: {}",
                entry.file.display(),
                entry.macro_name,
                entry.reason
            );
        }
        assert!(surface.unexpanded.iter().all(|e| !e.reason.is_empty()));
    }
}
