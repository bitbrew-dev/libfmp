//! Thin Python facade for `libfmp`.

mod client;
mod errors;
mod quote;

use pyo3::prelude::*;

#[pymodule]
mod _native {
    use pyo3::prelude::*;

    #[pymodule_init]
    fn init(module: &Bound<'_, PyModule>) -> PyResult<()> {
        module.add("__version__", libfmp::VERSION)?;
        let py = module.py();
        module.add("_FmpClient", py.get_type::<super::client::FmpClient>())?;
        super::errors::register(module)?;
        module.add("_QuoteShort", py.get_type::<super::quote::QuoteShort>())?;
        Ok(())
    }
}
