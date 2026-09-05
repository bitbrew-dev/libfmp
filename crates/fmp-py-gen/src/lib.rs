//! Shared library surface of the `fmp-py` generators.
//!
//! The crate ships two binaries: `gen_models` (response models, see
//! `src/bin/gen_models`) and `registry_check` (endpoint registry validation).
//! This library holds the pieces both the binaries and the later namespace
//! emitter consume, so the registry has one typed reader and one validator.

pub mod registry;
