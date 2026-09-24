//! Committed-source generator for the Go SDK at `sdk/go` (ADR 0030).
//!
//! Reads the endpoint registry, the wire contract of every `libfmp::Client`
//! method, and the response structs, and emits per domain
//! `sdk/go/<domain>_models.go` (models with shadow-struct decode) and
//! `sdk/go/<domain>.go` (namespace, queries, methods), plus the shared
//! `queries.go` (query types used by more than one domain) and
//! `namespaces.go` (the `Namespaces` struct Client embeds), and
//! `metadata_table.go` (the advisory `EndpointMetadata` of every method
//! whose descriptor attaches one, behind `EndpointMetadataFor`, and the call
//! paths serving each endpoint id, behind `EndpointMetadataByID`). Every file is
//! passed through `gofmt` before it is written, so a second run is a no-op.
//!
//! ```console
//! cargo run -p fmp-py-gen --bin gen_go -- --domain quote   # named domains
//! cargo run -p fmp-py-gen --bin gen_go                      # every generated domain
//! cargo run -p fmp-py-gen --bin gen_go -- --all             # every registry domain
//! ```
//!
//! A Rust type or arg kind the tables do not know fails the run naming the
//! struct and field (or the arg and the helper to add); nothing is emitted.

mod emit;
mod metadata;
mod methods;
mod models;
mod render;
#[cfg(test)]
mod tests;
mod types;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use fmp_py_gen::registry::wire::{WireSurface, wire_surface};
use fmp_py_gen::registry::{Registry, RegistryError, format_errors};
use fmp_py_gen::responses::{Discovery, StructDef, discover};

use crate::emit::{GENERATED_HEADER, gofmt};
use crate::metadata::render_metadata_table;
use crate::methods::{Context, DomainPlan, QueryPlan};
use crate::models::{plan_models, render_models};
use crate::render::{render_domain, render_namespaces, render_shared_queries};
use crate::types::TypeTable;

/// Which domains a run regenerates.
#[derive(Debug, PartialEq, Eq)]
enum Selection {
    /// `--domain <name>` repeated.
    Named(Vec<String>),
    /// No arguments: every domain whose `<domain>.go` carries the header.
    Generated,
    /// `--all`.
    All,
}

struct Args {
    selection: Selection,
    out_dir: Option<PathBuf>,
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut named = Vec::new();
    let mut all = false;
    let mut out_dir = None;
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--domain" => named.push(args.next().ok_or("--domain needs a name")?),
            "--all" => all = true,
            "--out-dir" => {
                out_dir = Some(PathBuf::from(args.next().ok_or("--out-dir needs a path")?))
            }
            other => {
                return Err(format!(
                    "unknown argument `{other}`; use --domain <name>, --all, --out-dir <dir>"
                ));
            }
        }
    }
    let selection = match (all, named.is_empty()) {
        (true, false) => return Err("--all and --domain are exclusive".to_string()),
        (true, true) => Selection::All,
        (false, false) => Selection::Named(named),
        (false, true) => Selection::Generated,
    };
    Ok(Args { selection, out_dir })
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sdk_go = args
        .out_dir
        .unwrap_or_else(|| manifest.join("../../sdk/go"));
    match run(&manifest, &sdk_go, &args.selection) {
        Ok(written) => {
            for file in &written {
                println!("wrote {}", file.display());
            }
            println!("gen_go ok: {} file(s) written", written.len());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            eprintln!("gen_go failed; nothing written");
            ExitCode::FAILURE
        }
    }
}

/// Loads every input, validates the registry, plans, renders, formats, and
/// only then writes. Returns the files written.
fn run(manifest: &Path, sdk_go: &Path, selection: &Selection) -> Result<Vec<PathBuf>, String> {
    let registry_dir = manifest.join("registry");
    let endpoints_root = manifest.join("../libfmp/src/endpoints");
    let responses_root = manifest.join("../libfmp/src/responses");
    let models_root = manifest.join("../fmp-py/src/models");

    let registry = Registry::load(&registry_dir).map_err(|error| match error {
        RegistryError::Invalid(errors) => format_errors(&errors),
        other => other.to_string(),
    })?;
    registry
        .validate(&endpoints_root, &models_root)
        .map_err(|errors| format_errors(&errors))?;
    let wire = wire_surface(&endpoints_root).map_err(|error| error.to_string())?;
    let discovery = discover(&responses_root).map_err(|error| error.to_string())?;

    let requested: BTreeSet<String> = match selection {
        Selection::Named(names) => {
            for name in names {
                if !registry.domains.iter().any(|domain| &domain.name == name) {
                    return Err(format!("`{name}` is not a registry domain"));
                }
            }
            names.iter().cloned().collect()
        }
        Selection::All => registry.domains.iter().map(|d| d.name.clone()).collect(),
        Selection::Generated => BTreeSet::new(),
    };
    let mut generated = generated_domains(&registry, sdk_go);
    generated.extend(requested.iter().cloned());
    if generated.is_empty() {
        return Err(
            "no domain selected and none is generated yet; pass --domain <name>".to_string(),
        );
    }
    let files = generate(&registry, &wire, &discovery, &generated, &requested)?;
    let mut formatted = Vec::with_capacity(files.len());
    for (name, source) in files {
        formatted.push((name, gofmt(&source)?));
    }
    let sdk_go = sdk_go
        .canonicalize()
        .unwrap_or_else(|_| sdk_go.to_path_buf());
    let mut written = Vec::new();
    for (name, source) in formatted {
        let path = sdk_go.join(name);
        fs::write(&path, source)
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?;
        written.push(path);
    }
    Ok(written)
}

/// Registry domains whose `<domain>.go` in `sdk_go` starts with the header.
fn generated_domains(registry: &Registry, sdk_go: &Path) -> BTreeSet<String> {
    registry
        .domains
        .iter()
        .filter(|domain| {
            fs::read_to_string(sdk_go.join(format!("{}.go", domain.name)))
                .is_ok_and(|source| source.starts_with(GENERATED_HEADER))
        })
        .map(|domain| domain.name.clone())
        .collect()
}

/// Renders every file of the run as `(file name, unformatted source)`: the
/// domain files of `requested` (all of `generated` when nothing is named)
/// and the three shared files derived from the whole `generated` set.
fn generate(
    registry: &Registry,
    wire: &WireSurface,
    discovery: &Discovery,
    generated: &BTreeSet<String>,
    requested: &BTreeSet<String>,
) -> Result<Vec<(String, String)>, String> {
    let model_owner: BTreeMap<String, String> = discovery
        .structs
        .iter()
        .filter_map(|def| Some((def.name.clone(), def.module_path.first()?.clone())))
        .collect();
    let context = Context {
        registry,
        wire,
        model_owner: &model_owner,
        generated,
    };
    let users = context.query_users()?;
    let queries = context.plan_queries()?;
    let table = TypeTable::new(&discovery.structs, &discovery.aliases);

    let mut plans: BTreeMap<String, (DomainPlan, String)> = BTreeMap::new();
    for domain in &registry.domains {
        if !generated.contains(&domain.name) {
            continue;
        }
        let defs: Vec<&StructDef> = discovery
            .structs
            .iter()
            .filter(|def| def.module_path.first() == Some(&domain.name))
            .collect();
        let models = plan_models(&domain.name, &defs, &table)?;
        let plan = context.plan_domain(domain, &queries, &users)?;
        plans.insert(
            domain.name.clone(),
            (plan, render_models(&domain.name, &models)),
        );
    }

    let shared: Vec<QueryPlan> = queries
        .values()
        .filter(|query| users.get(&query.name).is_some_and(|set| set.len() > 1))
        .cloned()
        .collect();
    let domain_plans: Vec<&DomainPlan> = plans.values().map(|(plan, _)| plan).collect();

    let mut files = Vec::new();
    for (name, (plan, models_source)) in &plans {
        if !requested.is_empty() && !requested.contains(name) {
            continue;
        }
        files.push((format!("{name}_models.go"), models_source.clone()));
        files.push((format!("{name}.go"), render_domain(plan)));
    }
    files.push(("queries.go".to_string(), render_shared_queries(&shared)));
    files.push((
        "namespaces.go".to_string(),
        render_namespaces(&domain_plans),
    ));
    files.push((
        "metadata_table.go".to_string(),
        render_metadata_table(&domain_plans)?,
    ));
    Ok(files)
}
