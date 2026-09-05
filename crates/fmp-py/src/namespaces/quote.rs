//! The `quote` endpoint namespace, reached from Python as `client.quote`.
//!
//! Each method releases the GIL, runs the async libfmp call on the shared
//! runtime, and maps the response rows into the generated typed models. The
//! hand-written shape here is the template the Phase 2 dual-emit macro follows.

use std::sync::Arc;

use libfmp::ClientBuilder;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::args;
use crate::errors::to_py_error;
use crate::models::quote::{Quote, QuoteShort};
use crate::runtime::block_on;

/// Quote endpoints for a single client, exposed as `client.quote`.
#[gen_stub_pyclass]
#[pyclass(module = "fmp._native.quote", frozen)]
pub(crate) struct QuoteNamespace {
    builder: Arc<ClientBuilder>,
}

impl QuoteNamespace {
    pub(crate) fn new(builder: Arc<ClientBuilder>) -> Self {
        Self { builder }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl QuoteNamespace {
    /// The full quote for a symbol.
    fn full(&self, py: Python<'_>, symbol: &str) -> PyResult<Vec<Quote>> {
        let query = args::ticker("symbol", symbol)?;
        let builder = self.builder.clone();
        let rows = py
            .detach(move || block_on(builder, |client| async move { client.quote(query).await }))?;
        rows.map(|items| items.into_iter().map(Quote::from).collect())
            .map_err(to_py_error)
    }

    /// The compact quote array for a symbol.
    fn short(&self, py: Python<'_>, symbol: &str) -> PyResult<Vec<QuoteShort>> {
        let query = args::ticker("symbol", symbol)?;
        let builder = self.builder.clone();
        let rows = py.detach(move || {
            block_on(
                builder,
                |client| async move { client.quote_short(query).await },
            )
        })?;
        rows.map(|items| items.into_iter().map(QuoteShort::from).collect())
            .map_err(to_py_error)
    }

    /// Compact quotes for every mutual fund, without a query.
    fn mutual_funds(&self, py: Python<'_>) -> PyResult<Vec<QuoteShort>> {
        let builder = self.builder.clone();
        let rows = py.detach(move || {
            block_on(builder, |client| async move {
                client.mutual_fund_quotes().await
            })
        })?;
        rows.map(|items| items.into_iter().map(QuoteShort::from).collect())
            .map_err(to_py_error)
    }
}
