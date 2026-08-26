//! Transport-neutral request execution contracts.

use std::{fmt, future::Future, pin::Pin};

use http::{Method, header::HeaderMap};
use reqwest::redirect::Policy;
use url::Url;

/// HTTP methods understood by endpoint descriptors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum HttpMethod {
    /// Retrieve a resource without a request body.
    Get,
}

impl HttpMethod {
    fn into_reqwest(self) -> Method {
        match self {
            Self::Get => Method::GET,
        }
    }
}

/// A fully prepared HTTP request passed to an injected executor.
///
/// Its `Debug` implementation intentionally omits the URL and every header
/// value because either can contain credentials.
#[derive(Clone)]
pub struct PreparedRequest {
    method: HttpMethod,
    url: Url,
    headers: HeaderMap,
}

impl PreparedRequest {
    pub(crate) fn new(method: HttpMethod, url: Url, headers: HeaderMap) -> Self {
        Self {
            method,
            url,
            headers,
        }
    }

    /// Returns the HTTP method.
    pub fn method(&self) -> HttpMethod {
        self.method
    }

    /// Explicitly exposes the request URL to an executor.
    ///
    /// Callers must treat the returned URL as secret-bearing diagnostic data.
    pub fn expose_url(&self) -> &Url {
        &self.url
    }

    /// Explicitly exposes request headers to an executor.
    ///
    /// Callers must honor `HeaderValue::is_sensitive` and must not log values.
    pub fn expose_headers(&self) -> &HeaderMap {
        &self.headers
    }
}

impl fmt::Debug for PreparedRequest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header_names: Vec<_> = self.headers.keys().map(|name| name.as_str()).collect();
        formatter
            .debug_struct("PreparedRequest")
            .field("method", &self.method)
            .field("url", &"[REDACTED URL]")
            .field("header_names", &header_names)
            .finish()
    }
}

/// A buffered HTTP response returned by an executor.
#[derive(Clone)]
pub struct TransportResponse {
    status: u16,
    headers: HeaderMap,
    body: Vec<u8>,
}

impl TransportResponse {
    /// Creates a buffered response for a custom or mock executor.
    pub fn new(status: u16, headers: HeaderMap, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            headers,
            body: body.into(),
        }
    }

    /// Returns the HTTP status code.
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Returns the response headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Returns the buffered response body.
    pub fn body(&self) -> &[u8] {
        &self.body
    }
}

impl fmt::Debug for TransportResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportResponse")
            .field("status", &self.status)
            .field("body_bytes", &self.body.len())
            .finish_non_exhaustive()
    }
}

/// A deliberately opaque executor failure.
///
/// Provider URLs and lower-level error strings are not retained because they
/// can contain query credentials.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutorError;

impl ExecutorError {
    /// Creates an opaque execution failure.
    pub const fn new() -> Self {
        Self
    }
}

impl Default for ExecutorError {
    fn default() -> Self {
        Self::new()
    }
}

/// Future returned by a dependency-injected HTTP executor.
pub type ExecutorFuture<'a> = Pin<
    Box<dyn Future<Output = std::result::Result<TransportResponse, ExecutorError>> + Send + 'a>,
>;

/// Object-safe async boundary used by the shared client transport.
pub trait HttpExecutor: fmt::Debug + Send + Sync {
    /// Executes one already-prepared request without following redirects.
    fn execute(&self, request: PreparedRequest) -> ExecutorFuture<'_>;
}

pub(crate) struct ReqwestExecutor {
    client: reqwest::Client,
}

impl ReqwestExecutor {
    pub(crate) fn new(
        timeout: std::time::Duration,
        connect_timeout: std::time::Duration,
    ) -> std::result::Result<Self, ExecutorError> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .connect_timeout(connect_timeout)
            // Redirects are handled above the executor boundary so every auth
            // mode follows exactly the same same-origin rule.
            .redirect(Policy::none())
            .build()
            .map_err(|_| ExecutorError)?;
        Ok(Self { client })
    }
}

impl fmt::Debug for ReqwestExecutor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ReqwestExecutor(..)")
    }
}

impl HttpExecutor for ReqwestExecutor {
    fn execute(&self, request: PreparedRequest) -> ExecutorFuture<'_> {
        Box::pin(async move {
            let response = self
                .client
                .request(request.method.into_reqwest(), request.url)
                .headers(request.headers)
                .send()
                .await
                .map_err(|_| ExecutorError)?;
            let status = response.status().as_u16();
            let headers = response.headers().clone();
            let body = response.bytes().await.map_err(|_| ExecutorError)?.to_vec();
            Ok(TransportResponse::new(status, headers, body))
        })
    }
}
