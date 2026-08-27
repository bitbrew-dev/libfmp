//! Thin Python facade for `libfmp`.

mod client;
mod errors;
mod quote;

use pyo3::prelude::*;

fn register_submodule<'py>(
    extension: &Bound<'py, PyModule>,
    package: &Bound<'py, PyAny>,
    sys_modules: &Bound<'py, pyo3::types::PyDict>,
    public_name: &str,
    module: &Bound<'py, PyModule>,
) -> PyResult<()> {
    extension.add_submodule(module)?;
    sys_modules.set_item(public_name, module)?;
    let attribute = public_name
        .rsplit_once('.')
        .map_or(public_name, |(_, name)| name);
    package.setattr(attribute, module)?;
    Ok(())
}

#[pymodule]
mod _native {
    use pyo3::{exceptions::PyRuntimeError, prelude::*, types::PyDict};

    #[pymodule_init]
    fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add("__version__", libfmp::VERSION)?;

        let py = module.py();
        let sys_modules = PyModule::import(py, "sys")?
            .getattr("modules")?
            .cast_into::<PyDict>()?;
        let package = sys_modules
            .get_item("fmp")?
            .ok_or_else(|| PyRuntimeError::new_err("fmp package is not initialized"))?;

        let client = PyModule::new(py, "client")?;
        client.add_class::<super::client::FmpClient>()?;
        client.add("__all__", vec!["FmpClient"])?;
        super::register_submodule(module, &package, &sys_modules, "fmp.client", &client)?;

        let errors = PyModule::new(py, "errors")?;
        super::errors::register(&errors)?;
        super::register_submodule(module, &package, &sys_modules, "fmp.errors", &errors)?;

        let quote = PyModule::new(py, "quote")?;
        quote.add_class::<super::quote::QuoteShort>()?;
        quote.add("__all__", vec!["QuoteShort"])?;
        super::register_submodule(module, &package, &sys_modules, "fmp.quote", &quote)?;
        Ok(())
    }
}
