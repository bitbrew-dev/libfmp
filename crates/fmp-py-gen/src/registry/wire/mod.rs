//! The wire contract behind every `libfmp::Client` method: endpoint id,
//! relative path, HTTP method, response contract, and the ordered query
//! parameters, read with `syn` from `crates/libfmp/src/endpoints/**`.
//!
//! The registry names methods, query types, and responses but carries no
//! request path and no parameter encoding; those live only in the Rust
//! sources, in each descriptor function's `EndpointSpec::get(id, path, ..)`
//! call and in each query type's `QueryParameters::encode` body. A Go
//! emitter needs both, so [`wire_surface`] resolves them per client method
//! and [`WireSurface::for_endpoint`] joins a registry entry to its wire
//! endpoint by the `libfmp` method name.
//!
//! Descriptor functions emitted by `macro_rules!` are recovered through
//! [`expand`](super::expand); helper chains (`forex_chart_light` calling
//! `asset_chart::chart_light` calling a generic `endpoint(path, ..)`) and
//! enum match tables (`IndexKind::constituent_path`) are evaluated, not
//! guessed. Whatever the evaluator cannot resolve is listed by method name
//! in [`WireSurface::unresolved`] with the reason, so `registry_check` can
//! report it.

mod collect;
mod params;
mod resolve;

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use super::Endpoint;
use super::scan::{Origin, ScanError, Unexpanded, collect_rust_files};

/// The HTTP method of an endpoint, mirroring `libfmp::transport::HttpMethod`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
}

impl HttpMethod {
    fn parse(variant: &str) -> Result<Self, String> {
        match variant {
            "Get" => Ok(Self::Get),
            other => Err(format!("unsupported `HttpMethod::{other}`")),
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Get => f.write_str("GET"),
        }
    }
}

/// How the response body is decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Contract {
    /// A JSON array of rows.
    Rows,
    /// A binary body with the listed acceptable content types.
    Binary(Vec<String>),
}

/// Whether the parameter is always sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Presence {
    Required,
    Optional,
}

/// Where a parameter's value comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// A field of the query struct, dotted through delegating builders
    /// (`assumptions.beta` for `self.assumptions.encode(encoder)`).
    Field(String),
    /// A literal written in the `encode` body (`encoder.required("short", true)`).
    Constant(String),
}

/// One query parameter in the order `encode` emits it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireParam {
    /// The exact wire key.
    pub name: String,
    pub presence: Presence,
    pub source: Source,
}

/// The wire contract of one `Client` method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WireEndpoint {
    /// The `libfmp::Client` method name, also the key in [`WireSurface`].
    pub method: String,
    /// The descriptor function the method calls, as `module::name`.
    pub descriptor: String,
    /// The file the descriptor function was found in.
    pub file: PathBuf,
    /// Whether the descriptor was written directly or emitted by a macro.
    pub origin: Origin,
    /// The stable endpoint id used in errors.
    pub id: String,
    /// The path relative to the client's base URL.
    pub relative_path: String,
    pub http_method: HttpMethod,
    /// The query type the descriptor encodes, which can differ from the
    /// registry's: a method taking no query may still send a constant
    /// (`mutual_fund_quotes` encodes `ShortOnlyQuery`). `None` for `()`.
    pub query_type: Option<String>,
    pub contract: Contract,
    pub params: Vec<WireParam>,
}

/// A client method the resolver could not map to literal wire facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unresolved {
    pub method: String,
    pub reason: String,
}

/// Every `Client` method's wire contract, keyed by method name, with the
/// methods that could not be resolved and the macros that did not expand.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct WireSurface {
    pub endpoints: BTreeMap<String, WireEndpoint>,
    pub unresolved: Vec<Unresolved>,
    pub unexpanded: Vec<Unexpanded>,
}

impl WireSurface {
    /// Builds the surface from already parsed files, with `endpoints_root`
    /// giving each file its module path.
    pub fn from_files(endpoints_root: &Path, files: &[(PathBuf, syn::File)]) -> WireSurface {
        let collected = collect::Collected::from_files(endpoints_root, files);
        let mut surface = WireSurface {
            unexpanded: collected.unexpanded.clone(),
            ..WireSurface::default()
        };
        for method in collected.client_methods.keys() {
            match resolve::resolve(&collected, method) {
                Ok(endpoint) => {
                    surface.endpoints.insert(method.clone(), endpoint);
                }
                Err(reason) => surface.unresolved.push(Unresolved {
                    method: method.clone(),
                    reason,
                }),
            }
        }
        surface
    }

    /// The wire endpoint behind a registry entry, joined by its `libfmp`
    /// method name.
    pub fn for_endpoint(&self, endpoint: &Endpoint) -> Option<&WireEndpoint> {
        self.endpoints.get(&endpoint.libfmp_method)
    }
}

/// Parses every `.rs` file under `endpoints_root` and resolves every
/// `Client` method's wire contract.
pub fn wire_surface(endpoints_root: &Path) -> Result<WireSurface, ScanError> {
    let mut paths = Vec::new();
    collect_rust_files(endpoints_root, &mut paths)?;
    paths.sort();
    let mut files = Vec::with_capacity(paths.len());
    for path in paths {
        let content = fs::read_to_string(&path).map_err(|source| ScanError::Io {
            path: path.clone(),
            source,
        })?;
        let parsed = syn::parse_file(&content).map_err(|source| ScanError::Syntax {
            path: path.clone(),
            source,
        })?;
        files.push((path, parsed));
    }
    Ok(WireSurface::from_files(endpoints_root, &files))
}
