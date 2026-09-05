//! Thin Python facade for `libfmp`.

pub mod args;
mod client;
mod errors;
mod models;
mod namespaces;
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
        super::register_namespaces(module)?;
        Ok(())
    }
}

/// Registers the domain submodules (for example `fmp._native.quote`) under the
/// extension and publishes each in `sys.modules` so `import fmp._native.quote`
/// resolves. Phase 2 extends this to every domain; Phase 1 wires `quote`.
fn register_namespaces(parent: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = parent.py();

    let quote = PyModule::new(py, "quote")?;
    quote.add_class::<models::quote::Quote>()?;
    quote.add_class::<models::quote::QuoteShort>()?;
    quote.add_class::<namespaces::quote::QuoteNamespace>()?;
    add_submodule(parent, "fmp._native.quote", &quote)?;

    let chart = PyModule::new(py, "chart")?;
    chart.add_class::<models::chart::StockChartLightBar>()?;
    chart.add_class::<models::chart::StockChartIntradayBar>()?;
    add_submodule(parent, "fmp._native.chart", &chart)?;

    Ok(())
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
