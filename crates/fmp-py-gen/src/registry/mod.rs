//! The declarative endpoint registry: one TOML file per domain under
//! `crates/fmp-py-gen/registry/`, read into a typed model and validated
//! against the real `libfmp` signatures.
//!
//! The file stem is the domain name. Entries at the top level belong to the
//! domain's root namespace; `[[namespace]]` sections carry a dotted `path`
//! (`statements.income`) for the nested layout the statements domain uses.
//!
//! ```toml
//! [[endpoint]]
//! name = "full"
//! method = "quote"
//! query = "QuoteQuery"
//! response = "quote::Quote"
//! doc = "The full quote for a symbol."
//! args = [{ name = "symbol", kind = "ticker" }]
//!
//! [[namespace]]
//! path = "statements.income"
//!
//! [[namespace.endpoint]]
//! name = "statement"
//! method = "income_statement"
//! query = "IncomeStatementQuery"
//! response = "statements::income::IncomeStatement"
//! doc = "Income statements for a symbol."
//! args = [
//!     { name = "symbol", kind = "ticker" },
//!     { name = "limit", kind = "limit", required = false },
//! ]
//! setters = [{ arg = "limit" }]
//! ```
//!
//! A constructor argument that is itself a builder type (`DcfAssumptions`
//! for `CustomDcfQuery::new(symbol, assumptions)`) is declared `nested`: its
//! setters become keyword-only parameters of the Python method and the
//! emitter assembles the builder before calling the constructor.
//!
//! ```toml
//! args = [
//!     { name = "symbol", kind = "ticker" },
//!     { name = "assumptions", nested = { type = "DcfAssumptions", setters = [
//!         { arg = "beta", kind = "finite_decimal" },
//!     ] } },
//! ]
//! ```

pub mod expand;
mod kinds;
pub mod naming;
pub mod scan;
mod schema;
mod validate;

use std::fmt;
use std::path::{Path, PathBuf};

pub use kinds::{ArgKind, UnknownKind};
pub use validate::{Report, Trusted, Verified};

/// Every domain file found in a registry directory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registry {
    pub domains: Vec<Domain>,
}

/// One `<domain>.toml` file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Domain {
    /// The file stem, also the first segment of every namespace path.
    pub name: String,
    /// The file the domain was read from, kept for error messages.
    pub file: PathBuf,
    pub namespaces: Vec<Namespace>,
}

/// One Python namespace object, addressed by its full dotted path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Namespace {
    /// The path segments, for example `["statements", "income"]`.
    pub path: Vec<String>,
    pub endpoints: Vec<Endpoint>,
}

/// One Python method bound to one `libfmp::Client` method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub python_name: String,
    pub libfmp_method: String,
    /// The query type the client method accepts; `None` for query-less methods.
    pub query_type: Option<String>,
    /// The generated model the rows map into; `None` for binary responses.
    pub response_model: Option<ModelPath>,
    pub doc: String,
    /// Every Python parameter in declaration order, the setter arguments of
    /// a nested builder flattened in at the builder's position.
    pub args: Vec<Arg>,
    /// Optional parameters applied through `with_*` builder methods.
    pub setters: Vec<Setter>,
    /// Builder types passed whole to the query constructor.
    pub nested: Vec<NestedBuilder>,
    /// Whether the client method returns `BinaryResponse` rather than rows.
    pub binary: bool,
    /// Whether the rows are untyped `DynamicObject`s handed to Python as
    /// `dict`s (registry `response = "dynamic"`); no model is involved.
    pub dynamic: bool,
}

/// A generated model, for example `statements::income::IncomeStatement`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelPath {
    /// Module segments under `crates/fmp-py/src/models/`.
    pub module: Vec<String>,
    /// The struct name.
    pub name: String,
}

/// One Python parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arg {
    pub name: String,
    pub kind: ArgKind,
    /// Required parameters are positional; optional ones default to `None`.
    pub required: bool,
}

/// An optional parameter routed through a query builder method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Setter {
    /// The name of the entry in [`Endpoint::args`] this setter consumes.
    pub arg: String,
    /// The builder method on the query type, `with_<arg>` unless overridden.
    pub method: String,
}

/// A builder type passed whole to the query constructor; its setters are
/// flattened into keyword-only parameters of the Python method.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NestedBuilder {
    /// The constructor parameter name, also the Rust local the emitter builds.
    pub name: String,
    /// The builder type, which needs a zero-parameter `pub fn new`.
    pub type_name: String,
    /// The index in [`Endpoint::args`] of the first flattened setter
    /// argument: where the builder sits among the constructor arguments.
    pub position: usize,
    /// The builder's setters, each consuming one flattened entry of
    /// [`Endpoint::args`].
    pub setters: Vec<Setter>,
}

/// One value passed to the query constructor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CtorArg<'a> {
    Plain(&'a Arg),
    Nested(&'a NestedBuilder),
}

impl<'a> CtorArg<'a> {
    /// The registry name of the constructor slot.
    pub fn name(self) -> &'a str {
        match self {
            CtorArg::Plain(arg) => &arg.name,
            CtorArg::Nested(builder) => &builder.name,
        }
    }

    /// The Rust local the emitter passes to the constructor.
    pub fn local(self) -> String {
        match self {
            CtorArg::Plain(arg) => arg.python_name(),
            CtorArg::Nested(builder) => builder.name.clone(),
        }
    }

    /// The `libfmp` type the constructor parameter must have.
    pub fn libfmp_type(self) -> &'a str {
        match self {
            CtorArg::Plain(arg) => arg.kind.libfmp_type(),
            CtorArg::Nested(builder) => &builder.type_name,
        }
    }
}

impl Endpoint {
    /// The values passed to the query constructor, in order: every argument
    /// no setter consumes, with each nested builder at its position.
    pub fn ctor_args(&self) -> Vec<CtorArg<'_>> {
        let nested_at = |index: usize| {
            self.nested
                .iter()
                .filter(move |builder| builder.position == index)
                .map(CtorArg::Nested)
        };
        let mut out = Vec::new();
        for (index, arg) in self.args.iter().enumerate() {
            out.extend(nested_at(index));
            if !self.consumed_by_setter(&arg.name) {
                out.push(CtorArg::Plain(arg));
            }
        }
        out.extend(nested_at(self.args.len()));
        out
    }

    /// Whether a query setter or a nested builder setter consumes `arg`.
    fn consumed_by_setter(&self, arg: &str) -> bool {
        self.setters.iter().any(|setter| setter.arg == arg)
            || self
                .nested
                .iter()
                .any(|builder| builder.setters.iter().any(|setter| setter.arg == arg))
    }

    /// The argument a setter consumes. Loading guarantees it exists.
    pub fn setter_arg(&self, setter: &Setter) -> Option<&Arg> {
        self.args.iter().find(|arg| arg.name == setter.arg)
    }
}

impl Arg {
    /// The parameter name as Python sees it: the registry name with a
    /// trailing `_` when that name is a Python keyword (`from` -> `from_`).
    pub fn python_name(&self) -> String {
        naming::python_safe_ident(&self.name)
    }
}

impl Namespace {
    /// The dotted form of the path, for example `statements.income`.
    pub fn dotted(&self) -> String {
        self.path.join(".")
    }
}

impl fmt::Display for ModelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for segment in &self.module {
            write!(f, "{segment}::")?;
        }
        f.write_str(&self.name)
    }
}

/// A problem with one registry entry, naming the file and the entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    pub file: PathBuf,
    /// The dotted Python path of the entry (`quote.full`), or the namespace
    /// path when the problem is not tied to one endpoint.
    pub entry: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(file: &Path, entry: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            file: file.to_path_buf(),
            entry: entry.into(),
            message: message.into(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: `{}`: {}",
            self.file.display(),
            self.entry,
            self.message
        )
    }
}

impl std::error::Error for ValidationError {}

/// Why a registry directory could not be read into a [`Registry`].
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("cannot read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("{path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: Box<toml::de::Error>,
    },
    #[error("no `<domain>.toml` files under {0}")]
    Empty(PathBuf),
    #[error("{}", format_errors(.0))]
    Invalid(Vec<ValidationError>),
}

/// Joins one error per line so a failing load reports every entry at once.
pub fn format_errors(errors: &[ValidationError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}
