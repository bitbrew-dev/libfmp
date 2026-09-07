//! Thin Python facade for `libfmp`.

pub mod args;
mod client;
mod client_namespaces;
mod errors;
mod facade;
mod facade_domains;
mod models;
mod namespaces;
mod registration;
mod runtime;

use pyo3::prelude::*;

#[pymodule]
mod _native {
    use pyo3::prelude::*;

    #[pymodule_init]
    fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add("__version__", libfmp::VERSION)?;
        module.add_class::<super::client::FmpClient>()?;
        super::errors::register(module)?;
        super::registration::register_namespaces(module)?;
        Ok(())
    }
}

/// Attaches `submodule` to `parent` and publishes it in `sys.modules` under
/// `path` so `import <path>` resolves at runtime.
fn add_submodule(
    parent: &Bound<'_, PyModule>,
    path: &str,
    submodule: &Bound<'_, PyModule>,
) -> PyResult<()> {
    parent.add_submodule(submodule)?;
    parent
        .py()
        .import("sys")?
        .getattr("modules")?
        .set_item(path, submodule)?;
    Ok(())
}

pyo3_stub_gen::define_stub_info_gatherer!(stub_info);
