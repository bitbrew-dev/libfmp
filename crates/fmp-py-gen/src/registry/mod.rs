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
    /// Every Python parameter, constructor arguments first in declaration order.
    pub args: Vec<Arg>,
    /// Optional parameters applied through `with_*` builder methods.
    pub setters: Vec<Setter>,
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

impl Endpoint {
    /// The arguments passed to the query constructor, in order: every
    /// argument no setter consumes.
    pub fn ctor_args(&self) -> impl Iterator<Item = &Arg> {
        self.args
            .iter()
            .filter(|arg| !self.setters.iter().any(|setter| setter.arg == arg.name))
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
