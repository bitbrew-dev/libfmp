//! Public-package facade declarations for `pyo3-stub-gen`.
//!
//! Every pyclass lives under `fmp._native.<domain>` (stub-gen requires the
//! native module path). The public import paths (`fmp`, `fmp.client`,
//! `fmp.errors`, `fmp.<domain>`) are pure-Python packages whose `__init__.py`
//! is written by `stub_gen` from `reexport_module_members!` declarations,
//! driven by `generate-init-py` in `pyproject.toml`.
//!
//! This file keeps the hand-picked top-level surface. The per-domain
//! packages (`fmp.quote`, `fmp.statements.income`, ...) are declared in the
//! generated `facade_domains.rs` with the wildcard form, which is safe
//! because the generated `registration.rs` adds every model the stub
//! inventory lists to its runtime module.

pyo3_stub_gen::module_variable!("fmp._native", "__version__", String);

pyo3_stub_gen::reexport_module_members!(
    "fmp" from "fmp._native";
    "FmpClient", "BinaryPayload", "__version__"
);
pyo3_stub_gen::reexport_module_members!("fmp" from "fmp._native.quote"; "QuoteShort");
pyo3_stub_gen::reexport_module_members!(
    "fmp" from "fmp._native.errors";
    "FmpError", "FmpValidationError", "FmpConfigError", "FmpTransportError", "FmpStatusError",
    "FmpDecodeError"
);
pyo3_stub_gen::reexport_module_members!("fmp.client" from "fmp._native"; "FmpClient");
pyo3_stub_gen::reexport_module_members!(
    "fmp.errors" from "fmp._native.errors";
    "FmpError", "FmpValidationError", "FmpConfigError", "FmpTransportError", "FmpStatusError",
    "FmpDecodeError"
);
