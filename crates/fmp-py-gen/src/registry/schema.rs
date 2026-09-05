//! The on-disk TOML shape and its lowering into the typed model.
//!
//! Lowering performs every check that needs no knowledge of `libfmp`:
//! identifier validity, namespace path prefixes, duplicate names, setter to
//! argument references, and the `binary` versus `response` exclusivity.
//! Problems are collected per file so one load reports them all.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{
    Arg, ArgKind, Domain, Endpoint, ModelPath, Namespace, Registry, RegistryError, Setter,
    ValidationError,
};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DomainFile {
    #[serde(default, rename = "endpoint")]
    endpoints: Vec<EndpointEntry>,
    #[serde(default, rename = "namespace")]
    namespaces: Vec<NamespaceSection>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NamespaceSection {
    path: String,
    #[serde(default, rename = "endpoint")]
    endpoints: Vec<EndpointEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EndpointEntry {
    name: String,
    method: String,
    query: Option<String>,
    response: Option<String>,
    doc: String,
    #[serde(default)]
    args: Vec<ArgEntry>,
    #[serde(default)]
    setters: Vec<SetterEntry>,
    #[serde(default)]
    binary: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArgEntry {
    name: String,
    kind: String,
    #[serde(default = "default_true")]
    required: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SetterEntry {
    arg: String,
    method: Option<String>,
}

const fn default_true() -> bool {
    true
}

impl Registry {
    /// Reads every `<domain>.toml` under `dir`, sorted by file name.
    ///
    /// Structural problems in any file are collected into
    /// [`RegistryError::Invalid`], each naming the file and entry.
    pub fn load(dir: &Path) -> Result<Registry, RegistryError> {
        let mut files: Vec<PathBuf> = fs::read_dir(dir)
            .map_err(|source| RegistryError::Io {
                path: dir.to_path_buf(),
                source,
            })?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
            .collect();
        files.sort();
        if files.is_empty() {
            return Err(RegistryError::Empty(dir.to_path_buf()));
        }

        let mut domains = Vec::new();
        let mut errors = Vec::new();
        for file in files {
            let content = fs::read_to_string(&file).map_err(|source| RegistryError::Io {
                path: file.clone(),
                source,
            })?;
            let parsed: DomainFile =
                toml::from_str(&content).map_err(|source| RegistryError::Parse {
                    path: file.clone(),
                    source: Box::new(source),
                })?;
            let name = file
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default();
            domains.push(lower_domain(name, &file, parsed, &mut errors));
        }
        if errors.is_empty() {
            Ok(Registry { domains })
        } else {
            Err(RegistryError::Invalid(errors))
        }
    }
}

fn lower_domain(
    name: String,
    file: &Path,
    parsed: DomainFile,
    errors: &mut Vec<ValidationError>,
) -> Domain {
    let mut namespaces = Vec::new();
    if !is_identifier(&name) {
        errors.push(ValidationError::new(
            file,
            &name,
            "domain file stem must be a Python identifier",
        ));
    }
    if !parsed.endpoints.is_empty() {
        namespaces.push(lower_namespace(
            file,
            vec![name.clone()],
            parsed.endpoints,
            errors,
        ));
    }
    for section in parsed.namespaces {
        let path: Vec<String> = section.path.split('.').map(str::to_owned).collect();
        let well_formed = path.iter().all(|segment| is_identifier(segment));
        if !well_formed || path.first() != Some(&name) {
            errors.push(ValidationError::new(
                file,
                &section.path,
                format!("namespace path must be `{name}` or `{name}.<sub>` identifiers"),
            ));
        }
        if namespaces
            .iter()
            .any(|existing: &Namespace| existing.path == path)
        {
            errors.push(ValidationError::new(
                file,
                &section.path,
                "duplicate namespace section",
            ));
        }
        namespaces.push(lower_namespace(file, path, section.endpoints, errors));
    }
    Domain {
        name,
        file: file.to_path_buf(),
        namespaces,
    }
}

fn lower_namespace(
    file: &Path,
    path: Vec<String>,
    entries: Vec<EndpointEntry>,
    errors: &mut Vec<ValidationError>,
) -> Namespace {
    let mut endpoints: Vec<Endpoint> = Vec::new();
    for entry in entries {
        let label = format!("{}.{}", path.join("."), entry.name);
        if endpoints.iter().any(|e| e.python_name == entry.name) {
            errors.push(ValidationError::new(
                file,
                &label,
                "duplicate endpoint name",
            ));
        }
        endpoints.push(lower_endpoint(file, &label, entry, errors));
    }
    Namespace { path, endpoints }
}

fn lower_endpoint(
    file: &Path,
    label: &str,
    entry: EndpointEntry,
    errors: &mut Vec<ValidationError>,
) -> Endpoint {
    let mut fail = |message: String| errors.push(ValidationError::new(file, label, message));
    if !is_identifier(&entry.name) {
        fail(format!("`{}` is not a Python identifier", entry.name));
    }
    if !is_identifier(&entry.method) {
        fail(format!("`{}` is not a Rust method name", entry.method));
    }
    if entry.doc.trim().is_empty() {
        fail("doc must not be empty".to_owned());
    }
    if entry.query.is_none() && !(entry.args.is_empty() && entry.setters.is_empty()) {
        fail("args and setters need a `query` type".to_owned());
    }
    let response_model = match (&entry.response, entry.binary) {
        (Some(_), true) => {
            fail("`binary = true` entries must not name a `response` model".to_owned());
            None
        }
        (None, false) => {
            fail("a `response` model is required unless `binary = true`".to_owned());
            None
        }
        (None, true) => None,
        (Some(response), false) => match parse_model_path(response) {
            Some(path) => Some(path),
            None => {
                fail(format!(
                    "`{response}` is not a `<module>::<Struct>` model path"
                ));
                None
            }
        },
    };

    let mut args = Vec::new();
    for arg in entry.args {
        if !is_identifier(&arg.name) {
            fail(format!("arg `{}` is not a Python identifier", arg.name));
        }
        if args.iter().any(|existing: &Arg| existing.name == arg.name) {
            fail(format!("duplicate arg `{}`", arg.name));
        }
        let kind = match arg.kind.parse::<ArgKind>() {
            Ok(kind) => kind,
            Err(error) => {
                fail(format!("arg `{}`: {error}", arg.name));
                ArgKind::Ticker
            }
        };
        args.push(Arg {
            name: arg.name,
            kind,
            required: arg.required,
        });
    }

    let mut setters = Vec::new();
    for setter in entry.setters {
        match args.iter().find(|arg| arg.name == setter.arg) {
            None => fail(format!("setter `{}` names no arg", setter.arg)),
            Some(arg) if arg.required => {
                fail(format!(
                    "setter arg `{}` must be `required = false`",
                    arg.name
                ));
            }
            Some(_) => {}
        }
        if setters
            .iter()
            .any(|existing: &Setter| existing.arg == setter.arg)
        {
            fail(format!("duplicate setter for `{}`", setter.arg));
        }
        let method = setter
            .method
            .unwrap_or_else(|| format!("with_{}", setter.arg));
        if !is_identifier(&method) {
            fail(format!(
                "setter method `{method}` is not a Rust method name"
            ));
        }
        setters.push(Setter {
            arg: setter.arg,
            method,
        });
    }

    Endpoint {
        python_name: entry.name,
        libfmp_method: entry.method,
        query_type: entry.query,
        response_model,
        doc: entry.doc,
        args,
        setters,
        binary: entry.binary,
    }
}

/// Parses `a::b::Name` into module segments and a struct name.
fn parse_model_path(value: &str) -> Option<ModelPath> {
    let mut segments: Vec<String> = value.split("::").map(str::to_owned).collect();
    let name = segments.pop()?;
    let is_struct = name.starts_with(|c: char| c.is_ascii_uppercase())
        && name.chars().all(|c| c.is_ascii_alphanumeric());
    if segments.is_empty() || !is_struct || !segments.iter().all(|s| is_identifier(s)) {
        return None;
    }
    Some(ModelPath {
        module: segments,
        name,
    })
}

/// A snake_case identifier valid in both Rust and Python.
fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(c) if c == '_' || c.is_ascii_lowercase())
        && chars.all(|c| c == '_' || c.is_ascii_lowercase() || c.is_ascii_digit())
}
