//! The wire contract behind every `libfmp::Client` method: endpoint id,
//! relative path, HTTP method, response contract, the ordered query
//! parameters, and the advisory metadata attached with `.with_metadata(..)`,
//! read with `syn` from `crates/libfmp/src/endpoints/**`.
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
mod metadata;
mod params;
mod resolve;

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

pub use metadata::{
    AccessRequirement, ConditionalPlanRequirement, DelayScope, EndpointBounds,
    GeographicAvailability, MarketDataDelay, PlanCondition, RealtimeAccess,
    UserDeclarationRequirement, WireMetadata,
};

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
    /// The advisory metadata the descriptor attaches with
    /// `.with_metadata(..)`, folded to plain data. `None` only when no call
    /// in the descriptor's helper chain attaches any; an unreadable
    /// argument leaves the method unresolved instead.
    pub metadata: Option<WireMetadata>,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::Registry;

    fn manifest() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    fn shipped() -> WireSurface {
        wire_surface(&manifest().join("../libfmp/src/endpoints")).expect("endpoints parse")
    }

    /// Builds a surface from `(relative path, source)` snippets under a
    /// virtual endpoints root.
    fn surface(snippets: &[(&str, &str)]) -> WireSurface {
        let root = Path::new("/virtual/endpoints");
        let files: Vec<(PathBuf, syn::File)> = snippets
            .iter()
            .map(|(path, source)| {
                (
                    root.join(path),
                    syn::parse_file(source).expect("snippet parses"),
                )
            })
            .collect();
        WireSurface::from_files(root, &files)
    }

    fn param(name: &str, presence: Presence, source: Source) -> WireParam {
        WireParam {
            name: name.to_owned(),
            presence,
            source,
        }
    }

    fn field(name: &str) -> Source {
        Source::Field(name.to_owned())
    }

    #[test]
    fn every_registry_method_resolves_to_one_path() {
        let surface = shipped();
        let registry = Registry::load(&manifest().join("registry")).expect("registry loads");
        let unresolved: Vec<String> = surface
            .unresolved
            .iter()
            .map(|entry| format!("{}: {}", entry.method, entry.reason))
            .collect();
        assert!(
            unresolved.is_empty(),
            "unresolved:\n{}",
            unresolved.join("\n")
        );
        assert!(surface.unexpanded.is_empty(), "{:?}", surface.unexpanded);
        let mut methods = 0;
        for domain in &registry.domains {
            for namespace in &domain.namespaces {
                for endpoint in &namespace.endpoints {
                    methods += 1;
                    let wire = surface.for_endpoint(endpoint).unwrap_or_else(|| {
                        panic!("{} has no wire endpoint", endpoint.libfmp_method)
                    });
                    assert!(!wire.relative_path.is_empty(), "{}", wire.method);
                    assert!(!wire.id.is_empty(), "{}", wire.method);
                    assert_eq!(wire.http_method, HttpMethod::Get, "{}", wire.method);
                    match (&wire.contract, endpoint.binary) {
                        (Contract::Rows, false) | (Contract::Binary(_), true) => {}
                        (contract, binary) => {
                            panic!(
                                "{}: contract {contract:?} vs binary = {binary}",
                                wire.method
                            )
                        }
                    }
                }
            }
        }
        assert_eq!(methods, 271);
        assert_eq!(surface.endpoints.len(), 271);
    }

    #[test]
    fn shipped_surface_covers_the_special_shapes() {
        let surface = shipped();
        let xlsx = &surface.endpoints["financial_reports_xlsx"];
        assert_eq!(xlsx.relative_path, "financial-reports-xlsx");
        assert_eq!(
            xlsx.contract,
            Contract::Binary(vec![
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_owned(),
                "application/octet-stream".to_owned(),
            ])
        );
        let senate = &surface.endpoints["senate_trades_by_name"];
        assert_eq!(senate.origin, Origin::Macro("name_endpoint".to_owned()));
        assert_eq!(senate.relative_path, "senate-trades-by-name");
        let macro_made: Vec<&str> = surface
            .endpoints
            .values()
            .filter(|endpoint| matches!(endpoint.origin, Origin::Macro(_)))
            .map(|endpoint| endpoint.method.as_str())
            .collect();
        assert_eq!(macro_made.len(), 8, "{macro_made:?}");
        let forex = &surface.endpoints["forex_chart_light"];
        assert_eq!(forex.descriptor, "forex::forex_chart_light");
        assert_eq!(forex.relative_path, "historical-price-eod/light");
        assert_eq!(forex.query_type.as_deref(), Some("AssetChartQuery"));
        let dow = &surface.endpoints["historical_dow_jones_constituents"];
        assert_eq!(dow.relative_path, "historical-dowjones-constituent");
        assert_eq!(dow.query_type, None);
        assert!(dow.params.is_empty());
        let funds = &surface.endpoints["mutual_fund_quotes"];
        assert_eq!(funds.query_type.as_deref(), Some("ShortOnlyQuery"));
        assert_eq!(
            funds.params,
            [param(
                "short",
                Presence::Required,
                Source::Constant("true".to_owned())
            )]
        );
        let dcf = &surface.endpoints["custom_discounted_cash_flow"];
        assert_eq!(
            dcf.params[0],
            param("symbol", Presence::Required, field("symbol"))
        );
        assert!(
            dcf.params
                .iter()
                .any(|p| p.source == field("assumptions.beta"))
        );
    }

    const CLIENT_PRELUDE: &str = r#"
        impl QueryParameters for () { fn encode(&self, _encoder: &mut QueryEncoder<'_>) {} }
    "#;

    #[test]
    fn resolves_a_direct_descriptor_and_its_encode_body() {
        let source = r#"
            pub struct ChartQuery { symbol: Ticker, from: Option<Date>, to: Option<Date> }
            impl QueryParameters for ChartQuery {
                fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                    encoder.required("symbol", &self.symbol);
                    encoder.optional("from", self.from);
                    encoder.optional("to", self.to.as_ref());
                }
            }
            const WORLDWIDE: EndpointMetadata =
                EndpointMetadata::new().with_geography(GeographicAvailability::Worldwide);
            pub fn chart(query: ChartQuery) -> EndpointSpec<ChartQuery, Vec<Bar>> {
                EndpointSpec::get("chart-id", "historical-chart/1min", query)
                    .with_metadata(WORLDWIDE)
                    .with_max_response_body_bytes(1)
            }
            impl Client {
                pub async fn chart(&self, query: impl Into<ChartQuery>) -> Result<Vec<Bar>> {
                    self.execute(&chart(query.into())).await
                }
            }
        "#;
        let surface = surface(&[("chart.rs", source)]);
        assert!(surface.unresolved.is_empty(), "{:?}", surface.unresolved);
        let chart = &surface.endpoints["chart"];
        assert_eq!(chart.descriptor, "chart::chart");
        assert_eq!(chart.origin, Origin::Direct);
        assert_eq!(chart.id, "chart-id");
        assert_eq!(chart.relative_path, "historical-chart/1min");
        assert_eq!(chart.http_method, HttpMethod::Get);
        assert_eq!(chart.query_type.as_deref(), Some("ChartQuery"));
        assert_eq!(chart.contract, Contract::Rows);
        assert_eq!(
            chart.params,
            [
                param("symbol", Presence::Required, field("symbol")),
                param("from", Presence::Optional, field("from")),
                param("to", Presence::Optional, field("to")),
            ]
        );
    }

    #[test]
    fn resolves_macro_emitted_queries_and_descriptors() {
        let source = r#"
            macro_rules! symbol_query {
                ($docs:literal, $query:ident) => {
                    #[doc = $docs]
                    pub struct $query { symbol: Ticker }
                    impl QueryParameters for $query {
                        fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                            encoder.required("symbol", &self.symbol);
                        }
                    }
                };
            }
            symbol_query!("docs", QuoteQuery);
            const US_ONLY: EndpointMetadata =
                EndpointMetadata::new().with_geography(GeographicAvailability::UsOnly);
            macro_rules! endpoint {
                ($function:ident, $path:literal, $query:ty) => {
                    #[doc = concat!("Describes `GET ", $path, "`.")]
                    pub fn $function(query: $query) -> EndpointSpec<$query, Vec<Row>> {
                        EndpointSpec::get($path, $path, query).with_metadata(US_ONLY)
                    }
                };
            }
            endpoint!(quote, "quote", QuoteQuery);
            impl Client {
                pub async fn quote(&self, query: impl Into<QuoteQuery>) -> Result<Vec<Row>> {
                    self.execute(&quote(query.into())).await
                }
            }
        "#;
        let surface = surface(&[("quote.rs", source)]);
        assert!(surface.unresolved.is_empty(), "{:?}", surface.unresolved);
        let quote = &surface.endpoints["quote"];
        assert_eq!(quote.origin, Origin::Macro("endpoint".to_owned()));
        assert_eq!(quote.relative_path, "quote");
        assert_eq!(
            quote.params,
            [param("symbol", Presence::Required, field("symbol"))]
        );
    }

    #[test]
    fn resolves_constants_delegation_and_binary_contracts() {
        let source = r#"
            pub const XLSX_TYPES: &[&str] = &["application/vnd.ms-excel", "application/octet-stream"];
            pub struct ShortOnlyQuery;
            impl QueryParameters for ShortOnlyQuery {
                fn encode(&self, encoder: &mut QueryEncoder<'_>) { encoder.required("short", true); }
            }
            pub struct Assumptions { beta: Option<FiniteDecimal> }
            impl Assumptions {
                fn encode(&self, encoder: &mut QueryEncoder<'_>) { encoder.optional("beta", self.beta); }
            }
            pub struct CustomQuery { symbol: Ticker, assumptions: Assumptions }
            impl QueryParameters for CustomQuery {
                fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                    encoder.required("symbol", &self.symbol);
                    self.assumptions.encode(encoder);
                }
            }
            pub fn quotes() -> EndpointSpec<ShortOnlyQuery, Vec<Row>> {
                EndpointSpec::get("batch-etf-quotes", "batch-etf-quotes", ShortOnlyQuery::new())
            }
            pub fn custom(query: CustomQuery) -> EndpointSpec<CustomQuery, Vec<Row>> {
                EndpointSpec::get("custom-dcf", "custom-discounted-cash-flow", query)
            }
            pub fn xlsx(query: CustomQuery) -> EndpointSpec<CustomQuery, BinaryResponse> {
                EndpointSpec::get_binary("xlsx", "financial-reports-xlsx", query, XLSX_TYPES)
            }
            pub fn explicit(query: CustomQuery) -> EndpointSpec<CustomQuery, BinaryResponse> {
                EndpointSpec::with_response(HttpMethod::Get, "x", "x-path", query, ResponseContract::binary(&["a/b"]))
            }
            impl Client {
                pub async fn quotes(&self) -> Result<Vec<Row>> { self.execute(&quotes()).await }
                pub async fn custom(&self, query: CustomQuery) -> Result<Vec<Row>> { self.execute(&custom(query)).await }
                pub async fn xlsx(&self, query: CustomQuery) -> Result<BinaryResponse> { self.execute(&xlsx(query)).await }
                pub async fn explicit(&self, query: CustomQuery) -> Result<BinaryResponse> { self.execute(&explicit(query)).await }
            }
        "#;
        let surface = surface(&[("dcf.rs", source)]);
        assert!(surface.unresolved.is_empty(), "{:?}", surface.unresolved);
        let quotes = &surface.endpoints["quotes"];
        assert_eq!(quotes.query_type.as_deref(), Some("ShortOnlyQuery"));
        assert_eq!(
            quotes.params,
            [param(
                "short",
                Presence::Required,
                Source::Constant("true".to_owned())
            )]
        );
        assert_eq!(
            surface.endpoints["custom"].params,
            [
                param("symbol", Presence::Required, field("symbol")),
                param("beta", Presence::Optional, field("assumptions.beta")),
            ]
        );
        assert_eq!(
            surface.endpoints["xlsx"].contract,
            Contract::Binary(vec![
                "application/vnd.ms-excel".to_owned(),
                "application/octet-stream".to_owned()
            ])
        );
        assert_eq!(
            surface.endpoints["explicit"].contract,
            Contract::Binary(vec!["a/b".to_owned()])
        );
        assert_eq!(surface.endpoints["explicit"].relative_path, "x-path");
    }

    #[test]
    fn resolves_helper_chains_and_enum_match_paths() {
        let asset_chart = r#"
            pub struct AssetChartQuery { symbol: Ticker }
            impl QueryParameters for AssetChartQuery {
                fn encode(&self, encoder: &mut QueryEncoder<'_>) { encoder.required("symbol", &self.symbol); }
            }
            const EOD_METADATA: EndpointMetadata =
                EndpointMetadata::new().with_bounds(EndpointBounds::new().with_response_rows(5_000));
            fn endpoint<R>(path: &'static str, query: AssetChartQuery, metadata: EndpointMetadata) -> EndpointSpec<AssetChartQuery, Vec<R>>
            where R: DeserializeOwned,
            {
                EndpointSpec::get(path, path, query).with_metadata(metadata)
            }
            pub(crate) fn chart_light(query: AssetChartQuery) -> EndpointSpec<AssetChartQuery, Vec<Bar>> {
                endpoint("historical-price-eod/light", query, EOD_METADATA)
            }
        "#;
        let forex = r#"
            pub fn forex_chart_light(query: AssetChartQuery) -> EndpointSpec<AssetChartQuery, Vec<Bar>> {
                super::asset_chart::chart_light(query)
            }
            impl Client {
                pub async fn forex_chart_light(&self, query: impl Into<AssetChartQuery>) -> Result<Vec<Bar>> {
                    self.execute(&forex_chart_light(query.into())).await
                }
            }
        "#;
        let indexes = r#"
            enum IndexKind { Sp500, Nasdaq }
            impl IndexKind {
                const fn constituent_path(self) -> &'static str {
                    match self {
                        Self::Sp500 => "sp500-constituent",
                        Self::Nasdaq => "nasdaq-constituent",
                    }
                }
            }
            fn constituents(kind: IndexKind) -> EndpointSpec<(), Vec<Row>> {
                let path = kind.constituent_path();
                EndpointSpec::get(path, path, ())
            }
            pub fn nasdaq_constituents() -> EndpointSpec<(), Vec<Row>> {
                constituents(IndexKind::Nasdaq)
            }
            impl Client {
                pub async fn nasdaq_constituents(&self) -> Result<Vec<Row>> {
                    self.execute(&nasdaq_constituents()).await
                }
            }
        "#;
        let surface = surface(&[
            ("asset_chart.rs", asset_chart),
            ("forex.rs", forex),
            ("indexes.rs", indexes),
            ("mod.rs", CLIENT_PRELUDE),
        ]);
        assert!(surface.unresolved.is_empty(), "{:?}", surface.unresolved);
        let forex = &surface.endpoints["forex_chart_light"];
        assert_eq!(forex.descriptor, "forex::forex_chart_light");
        assert_eq!(forex.relative_path, "historical-price-eod/light");
        assert_eq!(
            forex.params,
            [param("symbol", Presence::Required, field("symbol"))]
        );
        let nasdaq = &surface.endpoints["nasdaq_constituents"];
        assert_eq!(nasdaq.relative_path, "nasdaq-constituent");
        assert_eq!(nasdaq.query_type, None);
        assert!(nasdaq.params.is_empty());
    }

    #[test]
    fn unresolvable_shapes_are_reported_by_method_not_guessed() {
        let source = r#"
            pub struct DynQuery { pairs: Vec<(String, String)> }
            impl QueryParameters for DynQuery {
                fn encode(&self, encoder: &mut QueryEncoder<'_>) {
                    for (name, value) in &self.pairs { encoder.required(name, value); }
                }
            }
            pub fn dynamic(query: DynQuery) -> EndpointSpec<DynQuery, Vec<Row>> {
                EndpointSpec::get("dyn", "dyn", query)
            }
            pub fn computed(query: DynQuery) -> EndpointSpec<DynQuery, Vec<Row>> {
                let path = format!("{}-path", "x");
                EndpointSpec::get(path.as_str(), path.as_str(), query)
            }
            impl Client {
                pub async fn dynamic(&self, query: DynQuery) -> Result<Vec<Row>> { self.execute(&dynamic(query)).await }
                pub async fn computed(&self, query: DynQuery) -> Result<Vec<Row>> { self.execute(&computed(query)).await }
                pub async fn missing(&self, query: DynQuery) -> Result<Vec<Row>> { self.execute(&missing(query)).await }
                pub async fn indirect(&self, query: DynQuery) -> Result<Vec<Row>> {
                    let spec = dynamic(query);
                    self.execute(&spec).await
                }
            }
        "#;
        let surface = surface(&[("search.rs", source)]);
        assert!(
            surface.endpoints.is_empty(),
            "{:?}",
            surface.endpoints.keys()
        );
        let reasons: BTreeMap<&str, &str> = surface
            .unresolved
            .iter()
            .map(|entry| (entry.method.as_str(), entry.reason.as_str()))
            .collect();
        assert_eq!(reasons.len(), 4);
        assert!(reasons["dynamic"].contains("unsupported encode statement in `DynQuery`"));
        assert!(
            reasons["computed"].contains("is not a literal"),
            "{}",
            reasons["computed"]
        );
        assert!(reasons["missing"].contains("not a free function"));
        assert!(reasons["indirect"].contains("client body is not"));
    }
}
