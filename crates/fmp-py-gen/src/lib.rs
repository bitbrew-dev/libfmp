//! Shared library surface of the `fmp-py` generators.
//!
//! The crate ships three binaries: `gen_models` (response models, see
//! `src/bin/gen_models`), `registry_check` (endpoint registry validation),
//! and `gen_namespaces` (namespace classes, see `src/bin/gen_namespaces`).
//! This library holds the pieces the binaries share, so the registry has one
//! typed reader and one validator.

pub mod registry;
