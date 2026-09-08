//! The Python-side result of a binary endpoint: the response body as `bytes`
//! plus the transport metadata `libfmp` validated for it.
//!
//! Generated namespace methods for `binary = true` registry entries map the
//! `libfmp::endpoints::BinaryResponse` they receive through
//! [`BinaryPayload::from`] after the runtime hand-off returns, so the copy into
//! a Python `bytes` object happens exactly once while the thread is attached.

use libfmp::endpoints::BinaryResponse;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyTuple};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

/// A binary endpoint response, such as an XLSX workbook.
///
/// `data` is the body as `bytes`, `content_type` the exact validated
/// `Content-Type` header value, and `content_disposition` the
/// `Content-Disposition` header (an attachment filename hint) when the
/// provider sent a valid one. Instances are immutable and picklable.
#[gen_stub_pyclass]
#[pyclass(module = "fmp._native", frozen)]
pub(crate) struct BinaryPayload {
    data: Py<PyBytes>,
    content_type: String,
    content_disposition: Option<String>,
}

impl BinaryPayload {
    /// Copies `response` into a Python `bytes` object owned by `py`.
    pub(crate) fn from_response(py: Python<'_>, response: BinaryResponse) -> Self {
        Self {
            data: PyBytes::new(py, response.as_bytes()).unbind(),
            content_type: response.content_type().to_owned(),
            content_disposition: response.content_disposition().map(str::to_owned),
        }
    }

    fn media_type(&self) -> &str {
        self.content_type
            .split_once(';')
            .map_or(self.content_type.as_str(), |(media_type, _)| media_type)
            .trim()
    }
}

impl From<BinaryResponse> for BinaryPayload {
    /// Attaches to the interpreter (cheap when the thread already is) to
    /// build the `bytes` object.
    fn from(response: BinaryResponse) -> Self {
        Python::attach(|py| Self::from_response(py, response))
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl BinaryPayload {
    #[new]
    #[pyo3(signature = (data, content_type, content_disposition=None))]
    fn new(data: Py<PyBytes>, content_type: String, content_disposition: Option<String>) -> Self {
        Self {
            data,
            content_type,
            content_disposition,
        }
    }

    /// The response body.
    #[getter]
    fn data(&self, py: Python<'_>) -> Py<PyBytes> {
        self.data.clone_ref(py)
    }

    /// The exact validated `Content-Type` header value.
    #[getter]
    fn content_type(&self) -> String {
        self.content_type.clone()
    }

    /// The `Content-Disposition` header value when the provider sent a valid one.
    #[getter]
    fn content_disposition(&self) -> Option<String> {
        self.content_disposition.clone()
    }

    /// The body length in bytes.
    #[getter]
    fn byte_len(&self, py: Python<'_>) -> usize {
        self.data.bind(py).as_bytes().len()
    }

    fn __len__(&self, py: Python<'_>) -> usize {
        self.byte_len(py)
    }

    /// Reports the size and media type only: the disposition is
    /// provider-controlled text and the body is opaque.
    fn __repr__(&self, py: Python<'_>) -> String {
        format!(
            "BinaryPayload(byte_len={}, content_type='{}')",
            self.byte_len(py),
            self.media_type()
        )
    }

    fn __getnewargs__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        let members: Vec<Bound<'py, PyAny>> = vec![
            self.data.bind(py).clone().into_any(),
            self.content_type.clone().into_bound_py_any(py)?,
            self.content_disposition.clone().into_bound_py_any(py)?,
        ];
        PyTuple::new(py, members)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use http::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
    use http::{HeaderMap, HeaderValue};
    use libfmp::config::Authentication;
    use libfmp::endpoints::statements::FinancialReportsXlsxQuery;
    use libfmp::query::{FiscalPeriod, Year};
    use libfmp::transport::{
        ExecutorError, ExecutorFuture, HttpExecutor, PreparedRequest, TransportResponse,
    };
    use libfmp::types::Ticker;
    use libfmp::{Client, endpoints::BinaryResponse};
    use pyo3::prelude::*;

    use super::BinaryPayload;
    use crate::args::testing::{init, with_py};

    const XLSX: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
    const DISPOSITION: &str = "attachment; filename=AAPL-2024-FY.xlsx";
    const WORKBOOK: &[u8] = b"PK\x03\x04\xff\xfe\x80\x00 not utf-8";

    /// Answers the first request with one canned response, then fails.
    #[derive(Debug)]
    struct OneResponse(Mutex<Option<TransportResponse>>);

    impl HttpExecutor for OneResponse {
        fn execute(&self, _request: PreparedRequest) -> ExecutorFuture<'_> {
            Box::pin(async move {
                self.0
                    .lock()
                    .map_err(|_| ExecutorError::new())?
                    .take()
                    .ok_or_else(ExecutorError::new)
            })
        }
    }

    /// Runs the real XLSX endpoint against an injected transport, the only
    /// way to obtain a `BinaryResponse` outside `libfmp`.
    fn workbook_response(
        content_type: &'static str,
        disposition: Option<&'static str>,
    ) -> BinaryResponse {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
        if let Some(disposition) = disposition {
            headers.insert(CONTENT_DISPOSITION, HeaderValue::from_static(disposition));
        }
        let response = TransportResponse::new(200, headers, WORKBOOK);
        let client = Client::builder()
            .authentication(Authentication::fmp_header("secret"))
            .executor(Arc::new(OneResponse(Mutex::new(Some(response)))))
            .build()
            .expect("client builds");
        let query = FinancialReportsXlsxQuery::new(
            Ticker::new("AAPL").expect("valid ticker"),
            Year(2024),
            FiscalPeriod::FullYear,
        );
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("runtime builds")
            .block_on(client.financial_reports_xlsx(query))
            .expect("the transport answers with a workbook")
    }

    #[test]
    fn from_response_copies_the_body_and_metadata() {
        let response = workbook_response(XLSX, Some(DISPOSITION));
        with_py(|py| {
            let payload = BinaryPayload::from_response(py, response);
            assert_eq!(payload.data.bind(py).as_bytes(), WORKBOOK);
            assert_eq!(payload.byte_len(py), WORKBOOK.len());
            assert_eq!(payload.content_type(), XLSX);
            assert_eq!(payload.content_disposition().as_deref(), Some(DISPOSITION));
            assert_eq!(
                payload.__repr__(py),
                format!(
                    "BinaryPayload(byte_len={}, content_type='{XLSX}')",
                    WORKBOOK.len()
                )
            );
        });
    }

    #[test]
    fn from_attaches_and_keeps_the_parameterized_content_type() {
        init();
        let response = workbook_response("application/octet-stream; source=proxy", None);
        let payload = BinaryPayload::from(response);
        with_py(|py| {
            let data: Vec<u8> = payload.data(py).bind(py).as_bytes().to_vec();
            assert_eq!(data, WORKBOOK);
            assert_eq!(
                payload.content_type(),
                "application/octet-stream; source=proxy"
            );
            assert_eq!(payload.content_disposition(), None);
            assert!(
                payload
                    .__repr__(py)
                    .ends_with("content_type='application/octet-stream')")
            );
            let args = payload.__getnewargs__(py).expect("pickle args");
            assert_eq!(args.len(), 3);
            assert!(args.get_item(2).expect("disposition slot").is_none());
        });
    }
}
