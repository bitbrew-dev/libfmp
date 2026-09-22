//! Validates the endpoint registry against the real `libfmp` signatures and
//! the generated `fmp-py` models.
//!
//! Run it from the repository root with
//! `cargo run -p fmp-py-gen --bin registry_check [<registry dir>]`. The
//! directory defaults to `crates/fmp-py-gen/registry`. Every entry is printed
//! with its outcome; hard errors are listed and the process exits non-zero.
//!
//! A second pass joins every entry to its wire contract (endpoint path and
//! query parameters read from the Rust descriptor functions) and prints
//! `wire ok: N resolved, M unresolved`; any unresolved entry is listed by
//! method with the reason and also fails the run. A final line,
//! `metadata ok: N descriptors carry advisory metadata`, counts the
//! resolved entries whose descriptor attaches `.with_metadata(..)`.

use std::path::PathBuf;
use std::process::ExitCode;

use fmp_py_gen::registry::wire::{WireSurface, wire_surface};
use fmp_py_gen::registry::{Registry, RegistryError, format_errors};

fn main() -> ExitCode {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let registry_dir = std::env::args()
        .nth(1)
        .map_or_else(|| manifest.join("registry"), PathBuf::from);
    let endpoints_root = manifest.join("../libfmp/src/endpoints");
    let models_root = manifest.join("../fmp-py/src/models");

    let registry = match Registry::load(&registry_dir) {
        Ok(registry) => registry,
        Err(RegistryError::Invalid(errors)) => {
            eprintln!("{}", format_errors(&errors));
            eprintln!("registry invalid: {} error(s)", errors.len());
            return ExitCode::FAILURE;
        }
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    match registry.validate(&endpoints_root, &models_root) {
        Ok(report) => {
            for line in report.lines() {
                println!("{line}");
            }
            println!(
                "registry ok: {} verified, {} trusted",
                report.verified.len(),
                report.trusted.len()
            );
        }
        Err(errors) => {
            eprintln!("{}", format_errors(&errors));
            eprintln!("registry invalid: {} error(s)", errors.len());
            return ExitCode::FAILURE;
        }
    }
    let wire = match wire_surface(&endpoints_root) {
        Ok(wire) => wire,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    check_wire(&registry, &wire)
}

/// Joins every registry entry to its wire endpoint and prints the summary.
fn check_wire(registry: &Registry, wire: &WireSurface) -> ExitCode {
    let mut resolved = 0;
    let mut with_metadata = 0;
    let mut unresolved = Vec::new();
    for domain in &registry.domains {
        for namespace in &domain.namespaces {
            for endpoint in &namespace.endpoints {
                let method = &endpoint.libfmp_method;
                if let Some(wire) = wire.for_endpoint(endpoint) {
                    resolved += 1;
                    if wire.metadata.is_some() {
                        with_metadata += 1;
                    }
                    continue;
                }
                let reason = wire
                    .unresolved
                    .iter()
                    .find(|entry| &entry.method == method)
                    .map_or_else(
                        || "libfmp `Client` has no such method".to_owned(),
                        |entry| entry.reason.clone(),
                    );
                unresolved.push(format!("unresolved {method}: {reason}"));
            }
        }
    }
    for entry in &wire.unexpanded {
        eprintln!(
            "unexpanded {}: {}: {}",
            entry.file.display(),
            entry.macro_name,
            entry.reason
        );
    }
    for line in &unresolved {
        eprintln!("{line}");
    }
    if unresolved.is_empty() {
        println!("wire ok: {resolved} resolved, 0 unresolved");
        println!("metadata ok: {with_metadata} descriptors carry advisory metadata");
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "wire incomplete: {resolved} resolved, {} unresolved",
            unresolved.len()
        );
        ExitCode::FAILURE
    }
}
