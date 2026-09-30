//! The on-disk TOML shape and its lowering into the typed model.
//!
//! Lowering performs every check that needs no knowledge of `libfmp`:
//! identifier validity, namespace path prefixes, duplicate names, setter to
//! argument references, and the `binary` versus `response` exclusivity.
//! `response = "dynamic"` marks an endpoint whose rows carry no typed model
//! (`Vec<DynamicObject>`) and reach Python as plain `dict`s; `single = true`
//! marks one typed model returned bare instead of a `Vec` of rows. An arg carrying
//! `nested` instead of `kind` is a builder passed to the constructor; its
//! setters are flattened into [`Endpoint::args`] as optional parameters.
//! Problems are collected per file so one load reports them all.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::{
    Arg, ArgKind, Domain, Endpoint, ModelPath, Namespace, NestedBuilder, Registry, RegistryError,
    Setter, ValidationError,
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
    /// A `<module>::<Struct>` model path, or the literal `dynamic` for rows
    /// that have no typed model and are returned as `dict`s.
    response: Option<String>,
    doc: String,
    #[serde(default)]
    args: Vec<ArgEntry>,
    #[serde(default)]
    setters: Vec<SetterEntry>,
    #[serde(default)]
    binary: bool,
    #[serde(default)]
    single: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ArgEntry {
    name: String,
    /// The conversion kind of a plain parameter; absent for a nested builder.
    kind: Option<String>,
    /// Whether a plain parameter is positional; a nested builder is always
    /// passed, so `required = false` on it is rejected.
    #[serde(default = "default_true")]
    required: bool,
    /// A builder type passed whole to the constructor.
    nested: Option<NestedEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NestedEntry {
    #[serde(rename = "type")]
    type_name: String,
    setters: Vec<NestedSetterEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NestedSetterEntry {
    arg: String,
    kind: String,
    method: Option<String>,
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
    let dynamic = entry.response.as_deref() == Some(DYNAMIC_RESPONSE) && !entry.binary;
    if entry.single && (entry.binary || dynamic) {
        fail("`single = true` needs a typed `response` model".to_owned());
    }
    let response_model = match (&entry.response, entry.binary) {
        (Some(_), true) => {
            fail("`binary = true` entries must not name a `response` model".to_owned());
            None
        }
        (None, false) => {
            fail(
                "a `response` model (or `response = \"dynamic\"`) is required unless `binary = true`"
                    .to_owned(),
            );
            None
        }
        (None, true) => None,
        (Some(_), false) if dynamic => None,
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

    let mut args: Vec<Arg> = Vec::new();
    let mut nested: Vec<NestedBuilder> = Vec::new();
    let mut taken: Vec<String> = Vec::new();
    for arg in entry.args {
        if !is_identifier(&arg.name) {
            fail(format!("arg `{}` is not a Python identifier", arg.name));
        }
        if taken.contains(&arg.name) {
            fail(format!("duplicate arg `{}`", arg.name));
        }
        taken.push(arg.name.clone());
        match (arg.kind, arg.nested) {
            (Some(kind), None) => {
                let kind = parse_kind(&arg.name, &kind).unwrap_or_else(|message| {
                    fail(message);
                    ArgKind::Ticker
                });
                args.push(Arg {
                    name: arg.name,
                    kind,
                    required: arg.required,
                });
            }
            (None, Some(builder)) => {
                if !arg.required {
                    fail(format!(
                        "nested builder `{}` is always passed to the constructor; drop `required = false`",
                        arg.name
                    ));
                }
                if !is_type_name(&builder.type_name) {
                    fail(format!(
                        "nested builder `{}`: `{}` is not a Rust type name",
                        arg.name, builder.type_name
                    ));
                }
                if builder.setters.is_empty() {
                    fail(format!(
                        "nested builder `{}` needs at least one setter",
                        arg.name
                    ));
                }
                let position = args.len();
                let mut setters = Vec::new();
                for setter in builder.setters {
                    if !is_identifier(&setter.arg) {
                        fail(format!("arg `{}` is not a Python identifier", setter.arg));
                    }
                    if taken.contains(&setter.arg) {
                        fail(format!("duplicate arg `{}`", setter.arg));
                    }
                    taken.push(setter.arg.clone());
                    let kind = parse_kind(&setter.arg, &setter.kind).unwrap_or_else(|message| {
                        fail(message);
                        ArgKind::Ticker
                    });
                    args.push(Arg {
                        name: setter.arg.clone(),
                        kind,
                        required: false,
                    });
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
                nested.push(NestedBuilder {
                    name: arg.name,
                    type_name: builder.type_name,
                    position,
                    setters,
                });
            }
            _ => fail(format!(
                "arg `{}` needs exactly one of `kind` or `nested`",
                arg.name
            )),
        }
    }

    let mut setters = Vec::new();
    for setter in entry.setters {
        let nested_setter = nested
            .iter()
            .any(|builder| builder.setters.iter().any(|s| s.arg == setter.arg));
        match args.iter().find(|arg| arg.name == setter.arg) {
            None => fail(format!("setter `{}` names no arg", setter.arg)),
            Some(arg) if arg.required => {
                fail(format!(
                    "setter arg `{}` must be `required = false`",
                    arg.name
                ));
            }
            Some(_) if nested_setter => fail(format!(
                "setter arg `{}` is already applied by a nested builder",
                setter.arg
            )),
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
        nested,
        binary: entry.binary,
        single: entry.single,
        dynamic,
    }
}

/// Parses a registry `kind`, naming the argument in the error message.
fn parse_kind(arg: &str, kind: &str) -> Result<ArgKind, String> {
    kind.parse::<ArgKind>()
        .map_err(|error| format!("arg `{arg}`: {error}"))
}

/// The `response` spelling for endpoints returning untyped `dict` rows.
const DYNAMIC_RESPONSE: &str = "dynamic";

/// Parses `a::b::Name` into module segments and a struct name.
fn parse_model_path(value: &str) -> Option<ModelPath> {
    let mut segments: Vec<String> = value.split("::").map(str::to_owned).collect();
    let name = segments.pop()?;
    if segments.is_empty() || !is_type_name(&name) || !segments.iter().all(|s| is_identifier(s)) {
        return None;
    }
    Some(ModelPath {
        module: segments,
        name,
    })
}

/// A CamelCase Rust struct name.
fn is_type_name(value: &str) -> bool {
    value.starts_with(|c: char| c.is_ascii_uppercase())
        && value.chars().all(|c| c.is_ascii_alphanumeric())
}

/// A snake_case identifier valid in both Rust and Python.
fn is_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(c) if c == '_' || c.is_ascii_lowercase())
        && chars.all(|c| c == '_' || c.is_ascii_lowercase() || c.is_ascii_digit())
}
