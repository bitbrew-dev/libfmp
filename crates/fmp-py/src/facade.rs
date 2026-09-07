//! Public-package facade declarations for `pyo3-stub-gen`.
//!
//! Every pyclass lives under `fmp._native.<domain>` (stub-gen requires the
//! native module path). The public import paths (`fmp`, `fmp.client`,
//! `fmp.quote`) are pure-Python packages whose `__init__.py` is written by
//! `stub_gen` from the `reexport_module_members!` declarations below, driven
//! by `generate-init-py` in `pyproject.toml`.
//!
//! Items are listed explicitly rather than with the wildcard form: the
//! wildcard resolves against the stub inventory, which includes models that
//! `register_namespaces` does not yet add to the runtime module, and the
//! generated import would fail at runtime.

pyo3_stub_gen::module_variable!("fmp._native", "__version__", String);

pyo3_stub_gen::reexport_module_members!("fmp" from "fmp._native"; "FmpClient", "__version__");
pyo3_stub_gen::reexport_module_members!("fmp" from "fmp._native.quote"; "QuoteShort");
pyo3_stub_gen::reexport_module_members!("fmp.client" from "fmp._native"; "FmpClient");
pyo3_stub_gen::reexport_module_members!(
    "fmp.quote" from "fmp._native.quote"; "Quote", "QuoteNamespace", "QuoteShort"
);
