//! Cross-checks every registry entry against the real `libfmp` client
//! signatures and the generated `fmp-py` model files.
//!
//! Hard errors name the file and entry. Checks that cannot be carried out
//! (a query type whose constructor `syn` cannot see) are reported as trusted
//! entries in the [`Report`] instead of failing, so the reader can tell a
//! proven entry from an assumed one.

use std::fs;
use std::path::Path;

use super::scan::{Origin, QueryApi, Returns, Surface, module_path};
use super::{Arg, Endpoint, ModelPath, Registry, ValidationError};

/// One entry that passed every check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verified {
    pub entry: String,
    pub method: String,
    /// How the query type was found, or `None` for query-less methods.
    pub query_origin: Option<Origin>,
    /// Where the query type is defined, as segments under
    /// `libfmp::endpoints` (`["statements"]` for `IncomeStatementQuery`);
    /// `None` for query-less methods and for trusted entries.
    pub query_module: Option<Vec<String>>,
}

/// One entry whose query constructor or setters could not be checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trusted {
    pub entry: String,
    pub reason: String,
}

/// The outcome of a validation that produced no hard errors.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Report {
    pub verified: Vec<Verified>,
    pub trusted: Vec<Trusted>,
}

impl Report {
    /// One line per entry, suitable for a terminal.
    pub fn lines(&self) -> Vec<String> {
        let mut lines: Vec<String> = self
            .verified
            .iter()
            .map(|entry| {
                let via = match &entry.query_origin {
                    Some(Origin::Direct) => "query verified (direct)".to_owned(),
                    Some(Origin::Macro(name)) => format!("query verified (macro `{name}!`)"),
                    None => "no query".to_owned(),
                };
                format!(
                    "ok       {} -> Client::{} [{via}]",
                    entry.entry, entry.method
                )
            })
            .collect();
        lines.extend(
            self.trusted
                .iter()
                .map(|entry| format!("trusted  {}: {}", entry.entry, entry.reason)),
        );
        lines
    }
}

impl Registry {
    /// Validates every entry against `crates/libfmp/src/endpoints` (as
    /// `endpoints_root`) and `crates/fmp-py/src/models` (as `models_root`).
    pub fn validate(
        &self,
        endpoints_root: &Path,
        models_root: &Path,
    ) -> Result<Report, Vec<ValidationError>> {
        let surface = Surface::scan(endpoints_root).map_err(|error| {
            vec![ValidationError::new(
                endpoints_root,
                "scan",
                error.to_string(),
            )]
        })?;
        let mut report = Report::default();
        let mut errors = Vec::new();
        for domain in &self.domains {
            for namespace in &domain.namespaces {
                for endpoint in &namespace.endpoints {
                    let entry = format!("{}.{}", namespace.dotted(), endpoint.python_name);
                    let mut checker = Checker {
                        file: &domain.file,
                        entry: &entry,
                        endpoint,
                        surface: &surface,
                        endpoints_root,
                        models_root,
                        errors: &mut errors,
                        report: &mut report,
                    };
                    checker.run();
                }
            }
        }
        if errors.is_empty() {
            Ok(report)
        } else {
            Err(errors)
        }
    }
}

/// The `libfmp` row type behind `response = "dynamic"`.
const DYNAMIC_ROW: &str = "DynamicObject";

struct Checker<'a> {
    file: &'a Path,
    entry: &'a str,
    endpoint: &'a Endpoint,
    surface: &'a Surface,
    endpoints_root: &'a Path,
    models_root: &'a Path,
    errors: &'a mut Vec<ValidationError>,
    report: &'a mut Report,
}

impl Checker<'_> {
    fn fail(&mut self, message: impl Into<String>) {
        self.errors
            .push(ValidationError::new(self.file, self.entry, message));
    }

    fn run(&mut self) {
        let method_name = &self.endpoint.libfmp_method;
        let Some(method) = self.surface.methods.get(method_name) else {
            self.fail(format!(
                "libfmp `Client` has no `pub async fn {method_name}`"
            ));
            return;
        };
        let before = self.errors.len();
        self.check_query_type(method.query.as_deref());
        self.check_returns(&method.returns);
        let (query_origin, query_module) = self.check_query_api().unzip();
        if self.errors.len() == before {
            self.report.verified.push(Verified {
                entry: self.entry.to_owned(),
                method: method_name.clone(),
                query_origin,
                query_module,
            });
        }
    }

    fn check_query_type(&mut self, actual: Option<&str>) {
        let method = &self.endpoint.libfmp_method;
        match (self.endpoint.query_type.as_deref(), actual) {
            (None, None) => {}
            (Some(declared), Some(actual)) if declared == actual => {}
            (Some(declared), Some(actual)) => self.fail(format!(
                "query type is `{declared}` but `Client::{method}` takes `{actual}`"
            )),
            (None, Some(actual)) => self.fail(format!(
                "`Client::{method}` takes `{actual}` but the entry declares no `query`"
            )),
            (Some(declared), None) => self.fail(format!(
                "`Client::{method}` takes no query but the entry declares `{declared}`"
            )),
        }
    }

    fn check_returns(&mut self, returns: &Returns) {
        let method = &self.endpoint.libfmp_method;
        match returns {
            Returns::Binary if self.endpoint.binary => {}
            Returns::Binary => self.fail(format!(
                "`Client::{method}` returns `BinaryResponse`; set `binary = true`"
            )),
            Returns::Rows(_) if self.endpoint.binary => self.fail(format!(
                "`Client::{method}` returns rows, not `BinaryResponse`; drop `binary = true`"
            )),
            Returns::Rows(row) if row == DYNAMIC_ROW => {
                if !self.endpoint.dynamic {
                    self.fail(format!(
                        "`Client::{method}` returns `Vec<{DYNAMIC_ROW}>`; set `response = \"dynamic\"`"
                    ));
                }
            }
            Returns::Rows(row) if self.endpoint.dynamic => self.fail(format!(
                "`Client::{method}` returns `Vec<{row}>`, not `Vec<{DYNAMIC_ROW}>`; name a `response` model instead of `dynamic`"
            )),
            Returns::Rows(row) => {
                if let Some(model) = self.endpoint.response_model.clone() {
                    self.check_model(row, &model);
                }
            }
            Returns::Other(ty) => self.fail(format!(
                "`Client::{method}` returns unsupported type `{ty}`"
            )),
        }
    }

    fn check_model(&mut self, row: &str, model: &ModelPath) {
        if model.name != row && self.alias_target(row).as_deref() != Some(model.name.as_str()) {
            self.fail(format!(
                "`Client::{}` returns `Vec<{row}>` but the response model is `{model}`",
                self.endpoint.libfmp_method
            ));
            return;
        }
        let relative = model.module.join("/");
        let candidates = [
            self.models_root.join(format!("{relative}.rs")),
            self.models_root.join(&relative).join("mod.rs"),
        ];
        let Some(source) = candidates
            .iter()
            .find_map(|path| fs::read_to_string(path).ok())
        else {
            self.fail(format!(
                "model file `{relative}.rs` does not exist under {}",
                self.models_root.display()
            ));
            return;
        };
        let name = &model.name;
        let declared = [
            format!("pub(crate) struct {name} "),
            format!("pub(crate) struct {name}{{"),
            format!("pub struct {name} "),
            format!("pub struct {name}{{"),
        ];
        if !declared.iter().any(|needle| source.contains(needle)) {
            self.fail(format!(
                "model file `{relative}.rs` has no `pub(crate) struct {name}`; regenerate with `cargo run -p fmp-py-gen --bin gen_models`"
            ));
        }
    }

    /// Resolves a `pub type {row} = path::Target;` alias declared anywhere
    /// under `crates/libfmp/src/responses` to the last segment of its target,
    /// so an endpoint returning an aliased row verifies against the model of
    /// the struct behind the alias.
    fn alias_target(&self, row: &str) -> Option<String> {
        let responses_root = self.endpoints_root.parent()?.join("responses");
        let mut files = Vec::new();
        collect_rust_files(&responses_root, &mut files);
        files.iter().find_map(|path| {
            let source = fs::read_to_string(path).ok()?;
            let file = syn::parse_file(&source).ok()?;
            file.items.into_iter().find_map(|item| match item {
                syn::Item::Type(alias) if alias.ident == row => match *alias.ty {
                    syn::Type::Path(target) => {
                        target.path.segments.last().map(|s| s.ident.to_string())
                    }
                    _ => None,
                },
                _ => None,
            })
        })
    }

    /// Checks constructor arguments and setters against the query type's
    /// API, returning how and where the type was found when it could be
    /// checked.
    fn check_query_api(&mut self) -> Option<(Origin, Vec<String>)> {
        let query = self.endpoint.query_type.as_deref()?;
        let Some(api) = self.surface.queries.get(query).cloned() else {
            self.trust(format!(
                "query type `{query}` is not defined under endpoints/; constructor and setters unverified"
            ));
            return None;
        };
        self.check_ctor(query, &api);
        self.check_setters(query, &api);
        Some((api.origin, module_path(self.endpoints_root, &api.file)))
    }

    fn check_ctor(&mut self, query: &str, api: &QueryApi) {
        let declared: Vec<Arg> = self.endpoint.ctor_args().cloned().collect();
        let Some(params) = &api.ctor else {
            if declared.is_empty() {
                self.trust(format!(
                    "`{query}` has no `pub fn new`; construction unverified"
                ));
            } else {
                self.fail(format!(
                    "`{query}` has no `pub fn new` to receive the declared ctor args"
                ));
            }
            return;
        };
        if params.len() != declared.len() {
            let names: Vec<&str> = params.iter().map(|p| p.name.as_str()).collect();
            self.fail(format!(
                "`{query}::new` takes {} parameter(s) ({}) but {} ctor arg(s) are declared",
                params.len(),
                names.join(", "),
                declared.len()
            ));
            return;
        }
        for (arg, param) in declared.iter().zip(params) {
            if arg.kind.libfmp_type() != param.base_type {
                self.fail(format!(
                    "ctor arg `{}` has kind `{}` (a `{}`) but `{query}::new` parameter `{}` is `{}`",
                    arg.name,
                    arg.kind,
                    arg.kind.libfmp_type(),
                    param.name,
                    param.base_type
                ));
            }
        }
    }

    fn check_setters(&mut self, query: &str, api: &QueryApi) {
        for setter in &self.endpoint.setters {
            let Some(arg) = self.endpoint.setter_arg(setter).cloned() else {
                continue;
            };
            let Some(param) = api.setters.get(&setter.method) else {
                self.fail(format!(
                    "`{query}` has no single-parameter `pub fn {}` for arg `{}`",
                    setter.method, arg.name
                ));
                continue;
            };
            if arg.kind.libfmp_type() != param.base_type {
                self.fail(format!(
                    "setter arg `{}` has kind `{}` (a `{}`) but `{query}::{}` takes `{}`",
                    arg.name,
                    arg.kind,
                    arg.kind.libfmp_type(),
                    setter.method,
                    param.base_type
                ));
            }
        }
    }

    fn trust(&mut self, reason: String) {
        self.report.trusted.push(Trusted {
            entry: self.entry.to_owned(),
            reason,
        });
    }
}

/// Appends every `.rs` file under `dir`, recursively, in directory order.
fn collect_rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut paths: Vec<_> = entries.flatten().map(|entry| entry.path()).collect();
    paths.sort();
    for path in paths {
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}
