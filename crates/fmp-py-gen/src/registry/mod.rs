//! The declarative endpoint registry: one TOML file per domain under
//! `crates/fmp-py-gen/registry/`, read into a typed model and validated
//! against the real `libfmp` signatures.

mod kinds;

pub use kinds::ArgKind;
