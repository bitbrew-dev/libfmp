use pyo3::prelude::*;

/// Python-owned compact quote value with documented snake_case attributes.
#[pyclass(module = "fmp_py.quote", frozen)]
pub(crate) struct QuoteShort {
    #[pyo3(get)]
    symbol: String,
    #[pyo3(get)]
    price: f64,
    #[pyo3(get)]
    change: f64,
    #[pyo3(get)]
    volume: u64,
}

#[pymethods]
impl QuoteShort {
    #[new]
    fn new(symbol: String, price: f64, change: f64, volume: u64) -> Self {
        Self {
            symbol,
            price,
            change,
            volume,
        }
    }

    fn __getnewargs__(&self) -> (String, f64, f64, u64) {
        (self.symbol.clone(), self.price, self.change, self.volume)
    }
}

impl From<libfmp::responses::quote::QuoteShort> for QuoteShort {
    fn from(value: libfmp::responses::quote::QuoteShort) -> Self {
        Self::new(
            value.symbol.into_inner(),
            value.price,
            value.change,
            value.volume,
        )
    }
}
