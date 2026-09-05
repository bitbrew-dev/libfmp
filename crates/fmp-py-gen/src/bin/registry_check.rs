//! Validates the endpoint registry against the real `libfmp` signatures and
//! the generated `fmp-py` models.
//!
//! Run it from the repository root with
//! `cargo run -p fmp-py-gen --bin registry_check [<registry dir>]`. The
//! directory defaults to `crates/fmp-py-gen/registry`. Every entry is printed
//! with its outcome; hard errors are listed and the process exits non-zero.

use std::path::PathBuf;
use std::process::ExitCode;

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
            ExitCode::SUCCESS
        }
        Err(errors) => {
            eprintln!("{}", format_errors(&errors));
            eprintln!("registry invalid: {} error(s)", errors.len());
            ExitCode::FAILURE
        }
    }
}
