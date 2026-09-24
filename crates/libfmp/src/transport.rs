//! Transport-neutral request execution contracts.

use std::{fmt, future::Future, pin::Pin};

use bytes::{Bytes, BytesMut};
use http::{Method, header::HeaderMap};
use reqwest::redirect::Policy;
use url::Url;

const INITIAL_RESPONSE_CAPACITY: usize = 8 * 1024;

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
    max_response_body_bytes: usize,
}

impl PreparedRequest {
    pub(crate) fn new(
        method: HttpMethod,
        url: Url,
        headers: HeaderMap,
        max_response_body_bytes: usize,
    ) -> Self {
        Self {
            method,
            url,
            headers,
            max_response_body_bytes,
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

    /// Returns the largest response body this request may buffer.
    ///
    /// Custom executors should reject a declared or streamed response body
    /// before it exceeds this value. The client also checks returned buffers.
    pub fn max_response_body_bytes(&self) -> usize {
        self.max_response_body_bytes
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
    body: Bytes,
    body_limit_exceeded: bool,
}

impl TransportResponse {
    /// Creates a buffered response for a custom or mock executor.
    pub fn new(status: u16, headers: HeaderMap, body: impl Into<Vec<u8>>) -> Self {
        Self::from_bytes(status, headers, Bytes::from(body.into()))
    }

    /// Creates a buffered response from an owned byte buffer without copying.
    pub fn from_bytes(status: u16, headers: HeaderMap, body: Bytes) -> Self {
        Self {
            status,
            headers,
            body,
            body_limit_exceeded: false,
        }
    }

    fn too_large(status: u16, headers: HeaderMap) -> Self {
        Self {
            status,
            headers,
            body: Bytes::new(),
            body_limit_exceeded: true,
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

    /// Clones the cheaply reference-counted response body buffer.
    pub fn body_bytes(&self) -> Bytes {
        self.body.clone()
    }

    pub(crate) fn body_limit_exceeded(&self) -> bool {
        self.body_limit_exceeded
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
/// can contain query credentials. Construct it with [`ExecutorError::new`];
/// the unit-struct literal is reserved so a reason can be added later.
///
/// ```compile_fail,E0603
/// let _ = libfmp::transport::ExecutorError;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
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

impl fmt::Display for ExecutorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("HTTP request execution failed")
    }
}

impl std::error::Error for ExecutorError {}

/// Future returned by a dependency-injected HTTP executor.
pub type ExecutorFuture<'a> = Pin<
    Box<dyn Future<Output = std::result::Result<TransportResponse, ExecutorError>> + Send + 'a>,
>;

/// Object-safe async boundary used by the shared client transport.
///
/// Future releases add methods to this trait only with default
/// implementations, so existing executors keep compiling.
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
            .map_err(|_| ExecutorError::new())?;
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
            let max_body_bytes = request.max_response_body_bytes;
            let mut response = self
                .client
                .request(request.method.into_reqwest(), request.url)
                .headers(request.headers)
                .send()
                .await
                .map_err(|_| ExecutorError::new())?;
            let status = response.status().as_u16();
            let headers = response.headers().clone();
            if is_redirect_status(status) {
                return Ok(TransportResponse::from_bytes(status, headers, Bytes::new()));
            }
            if response
                .content_length()
                .is_some_and(|length| length > max_body_bytes as u64)
            {
                return Ok(TransportResponse::too_large(status, headers));
            }
            let mut body = BytesMut::with_capacity(initial_response_capacity(
                response.content_length(),
                max_body_bytes,
            ));
            while let Some(chunk) = response.chunk().await.map_err(|_| ExecutorError::new())? {
                if body
                    .len()
                    .checked_add(chunk.len())
                    .is_none_or(|length| length > max_body_bytes)
                {
                    return Ok(TransportResponse::too_large(status, headers));
                }
                body.extend_from_slice(&chunk);
            }
            Ok(TransportResponse::from_bytes(
                status,
                headers,
                body.freeze(),
            ))
        })
    }
}

fn is_redirect_status(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}

fn initial_response_capacity(content_length: Option<u64>, max_body_bytes: usize) -> usize {
    content_length
        .and_then(|length| usize::try_from(length).ok())
        .unwrap_or(0)
        .min(max_body_bytes)
        .min(INITIAL_RESPONSE_CAPACITY)
}

#[cfg(test)]
mod tests {
    use super::{INITIAL_RESPONSE_CAPACITY, initial_response_capacity};

    #[test]
    fn declared_lengths_only_seed_a_small_bounded_allocation() {
        assert_eq!(initial_response_capacity(None, usize::MAX), 0);
        assert_eq!(initial_response_capacity(Some(17), usize::MAX), 17);
        assert_eq!(initial_response_capacity(Some(17), 8), 8);
        assert_eq!(
            initial_response_capacity(Some(64 * 1024 * 1024), usize::MAX),
            INITIAL_RESPONSE_CAPACITY
        );
    }
}
