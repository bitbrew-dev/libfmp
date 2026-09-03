//! Transport-neutral request execution contracts.

use std::{fmt, future::Future, pin::Pin};

use bytes::{Bytes, BytesMut};
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
pub struct ExecutorError {
    kind: ExecutorErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExecutorErrorKind {
    Other,
    ResponseTooLarge,
}

impl ExecutorError {
    /// Creates an opaque execution failure.
    pub const fn new() -> Self {
        Self {
            kind: ExecutorErrorKind::Other,
        }
    }

    /// Creates a safe failure indicating the configured body limit was exceeded.
    pub const fn response_too_large() -> Self {
        Self {
            kind: ExecutorErrorKind::ResponseTooLarge,
        }
    }

    /// Reports whether response buffering exceeded the request's byte limit.
    pub const fn is_response_too_large(self) -> bool {
        matches!(self.kind, ExecutorErrorKind::ResponseTooLarge)
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
            if response
                .content_length()
                .is_some_and(|length| length > max_body_bytes as u64)
            {
                return Err(ExecutorError::response_too_large());
            }
            let mut body = BytesMut::with_capacity(
                response
                    .content_length()
                    .and_then(|length| usize::try_from(length).ok())
                    .unwrap_or(0)
                    .min(max_body_bytes),
            );
            while let Some(chunk) = response.chunk().await.map_err(|_| ExecutorError::new())? {
                if body
                    .len()
                    .checked_add(chunk.len())
                    .is_none_or(|length| length > max_body_bytes)
                {
                    return Err(ExecutorError::response_too_large());
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
