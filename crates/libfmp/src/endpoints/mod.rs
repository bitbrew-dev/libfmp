//! Transport-independent endpoint, query, and response contracts.

pub mod analyst;
pub mod asset_chart;
pub mod calendar;
pub mod chart;
pub mod commodities;
pub mod company;
pub mod congressional;
pub mod crypto;
pub mod dcf;
pub mod directory;
pub mod economics;
pub mod esg;
pub mod forex;
pub mod funds;
pub mod indexes;
pub mod insider_trading;
pub mod institutional_ownership;
pub mod market;
pub mod market_hours;
pub mod metadata;
pub mod news;
pub mod quote;
pub mod screener;
pub mod search;
pub mod sec_filings;
pub mod statements;
pub mod technical_indicators;
pub mod transcripts;

use std::{fmt, marker::PhantomData};

use serde::de::DeserializeOwned;

use crate::transport::HttpMethod;
use metadata::EndpointMetadata;

/// The JSON media type requested by FMP's structured-data endpoints.
pub const APPLICATION_JSON: &str = "application/json";

/// A typed endpoint description that contains no transport configuration.
///
/// Domain endpoint modules own values of this type. Base URLs,
/// authentication, default headers, redirects, and timeouts stay on `Client`.
pub struct EndpointSpec<Q, R> {
    method: HttpMethod,
    id: &'static str,
    relative_path: &'static str,
    query: Q,
    response: ResponseContract<R>,
    metadata: EndpointMetadata,
}

impl<Q, R> EndpointSpec<Q, R> {
    /// Creates a descriptor with an explicit response contract.
    pub const fn with_response(
        method: HttpMethod,
        id: &'static str,
        relative_path: &'static str,
        query: Q,
        response: ResponseContract<R>,
    ) -> Self {
        Self {
            method,
            id,
            relative_path,
            query,
            response,
            metadata: EndpointMetadata::new(),
        }
    }

    /// Returns the stable logical endpoint identity used in errors.
    pub const fn id(&self) -> &'static str {
        self.id
    }

    /// Returns the relative endpoint path.
    pub const fn relative_path(&self) -> &'static str {
        self.relative_path
    }

    /// Returns the endpoint method.
    pub const fn method(&self) -> HttpMethod {
        self.method
    }

    /// Borrows the typed query value.
    pub const fn query(&self) -> &Q {
        &self.query
    }

    /// Borrows the response decoding and content-type contract.
    pub const fn response(&self) -> &ResponseContract<R> {
        &self.response
    }

    /// Returns documentation metadata associated with this endpoint.
    pub const fn metadata(&self) -> EndpointMetadata {
        self.metadata
    }

    /// Attaches additive documentation metadata without changing the endpoint contract.
    pub const fn with_metadata(mut self, metadata: EndpointMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

impl<Q, R> EndpointSpec<Q, R>
where
    R: DeserializeOwned,
{
    /// Creates a reusable GET descriptor for a JSON response.
    pub const fn get(id: &'static str, relative_path: &'static str, query: Q) -> Self {
        Self::with_response(
            HttpMethod::Get,
            id,
            relative_path,
            query,
            ResponseContract::json(),
        )
    }

    /// Creates a JSON descriptor with an explicit method.
    ///
    /// `get` is preferred for FMP's documented GET endpoints. This constructor
    /// remains available to transport adapters and existing callers.
    pub const fn new(
        method: HttpMethod,
        id: &'static str,
        relative_path: &'static str,
        query: Q,
    ) -> Self {
        Self::with_response(method, id, relative_path, query, ResponseContract::json())
    }
}

impl<Q> EndpointSpec<Q, BinaryBody> {
    /// Creates a reusable GET descriptor for a binary response.
    ///
    /// This is the response hook needed by download endpoints such as the XLSX
    /// endpoint documented in `outer.md`; it does not define such an endpoint.
    pub const fn get_binary(
        id: &'static str,
        relative_path: &'static str,
        query: Q,
        expected_content_types: &'static [&'static str],
    ) -> Self {
        Self::with_response(
            HttpMethod::Get,
            id,
            relative_path,
            query,
            ResponseContract::binary(expected_content_types),
        )
    }
}

impl<Q, R> fmt::Debug for EndpointSpec<Q, R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EndpointSpec")
            .field("method", &self.method)
            .field("id", &self.id)
            .field("relative_path", &self.relative_path)
            .field("query", &"[REDACTED QUERY]")
            .field("response", &self.response)
            .field("metadata", &self.metadata)
            .finish()
    }
}

/// An encoder used by typed endpoint queries to emit exact wire keys.
///
/// Values are supplied unescaped. URL percent-encoding happens once, when the
/// transport appends these pairs to the request URL.
pub struct QueryEncoder<'a> {
    visitor: &'a mut dyn FnMut(&str, &str),
}

impl<'a> QueryEncoder<'a> {
    pub(crate) fn new(visitor: &'a mut dyn FnMut(&str, &str)) -> Self {
        Self { visitor }
    }

    /// Emits a required query value under its exact wire key.
    pub fn required(&mut self, name: &'static str, value: impl fmt::Display) {
        let value = value.to_string();
        (self.visitor)(name, &value);
    }

    /// Emits an optional query value only when it is present.
    pub fn optional<T>(&mut self, name: &'static str, value: Option<T>)
    where
        T: fmt::Display,
    {
        if let Some(value) = value {
            self.required(name, value);
        }
    }
}

/// A typed query that emits unencoded values in deterministic wire order.
pub trait QueryParameters {
    /// Encodes required and present optional values using exact provider keys.
    fn encode(&self, encoder: &mut QueryEncoder<'_>);
}

impl QueryParameters for () {
    fn encode(&self, _encoder: &mut QueryEncoder<'_>) {}
}

impl QueryParameters for Vec<(String, String)> {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        for (name, value) in self {
            let value = value.as_str();
            (encoder.visitor)(name, value);
        }
    }
}

impl<const N: usize> QueryParameters for [(&str, &str); N] {
    fn encode(&self, encoder: &mut QueryEncoder<'_>) {
        for (name, value) in self {
            (encoder.visitor)(name, value);
        }
    }
}

/// Expected response media types for an endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExpectedContentType {
    /// Standard JSON and registered vendor JSON media types (`+json`).
    Json,
    /// One of the listed binary media types, compared case-insensitively.
    Binary(&'static [&'static str]),
}

impl ExpectedContentType {
    pub(crate) fn matches(self, content_type: &str) -> bool {
        let media_type = content_type
            .split_once(';')
            .map_or(content_type, |(media_type, _)| media_type)
            .trim();
        match self {
            Self::Json => {
                media_type.eq_ignore_ascii_case(APPLICATION_JSON)
                    || media_type
                        .to_ascii_lowercase()
                        .strip_suffix("+json")
                        .is_some_and(|prefix| prefix.contains('/'))
            }
            Self::Binary(expected) => expected
                .iter()
                .any(|expected| media_type.eq_ignore_ascii_case(expected)),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ResponseMetadata<'a> {
    content_type: &'a str,
    content_disposition: Option<&'a str>,
}

impl<'a> ResponseMetadata<'a> {
    pub(crate) const fn new(content_type: &'a str, content_disposition: Option<&'a str>) -> Self {
        Self {
            content_type,
            content_disposition,
        }
    }
}

type Decoder<R> = for<'a> fn(&[u8], ResponseMetadata<'a>) -> std::result::Result<R, ()>;

/// Decoding and media-type expectations associated with one endpoint.
pub struct ResponseContract<R> {
    expected_content_type: ExpectedContentType,
    decoder: Decoder<R>,
    response: PhantomData<fn() -> R>,
}

impl<R> ResponseContract<R> {
    /// Returns the media types accepted by this response contract.
    pub const fn expected_content_type(&self) -> ExpectedContentType {
        self.expected_content_type
    }

    pub(crate) fn decode(
        &self,
        body: &[u8],
        metadata: ResponseMetadata<'_>,
    ) -> std::result::Result<R, ()> {
        (self.decoder)(body, metadata)
    }
}

impl<R> ResponseContract<R>
where
    R: DeserializeOwned,
{
    /// Creates a contract that preserves the response's documented JSON shape.
    pub const fn json() -> Self {
        Self {
            expected_content_type: ExpectedContentType::Json,
            decoder: decode_json::<R>,
            response: PhantomData,
        }
    }
}

impl ResponseContract<BinaryBody> {
    /// Creates a raw binary contract restricted to documented media types.
    pub const fn binary(expected_content_types: &'static [&'static str]) -> Self {
        Self {
            expected_content_type: ExpectedContentType::Binary(expected_content_types),
            decoder: decode_binary,
            response: PhantomData,
        }
    }
}

impl<R> fmt::Debug for ResponseContract<R> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResponseContract")
            .field("expected_content_type", &self.expected_content_type)
            .field("response_type", &std::any::type_name::<R>())
            .finish()
    }
}

fn decode_json<R>(body: &[u8], _metadata: ResponseMetadata<'_>) -> std::result::Result<R, ()>
where
    R: DeserializeOwned,
{
    serde_json::from_slice(body).map_err(|_| ())
}

/// An owned binary response with validated media metadata.
///
/// `Content-Type` is retained exactly as received after the endpoint contract
/// validates its media type. A valid `Content-Disposition` value is retained
/// when supplied, but is never included in `Debug` output because it can carry
/// an opaque filename or other provider-controlled text.
#[derive(Clone, PartialEq, Eq)]
pub struct BinaryResponse {
    bytes: Vec<u8>,
    content_type: Box<str>,
    content_disposition: Option<Box<str>>,
}

impl BinaryResponse {
    /// Borrows the response bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the owned response bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Returns the exact validated `Content-Type` header value.
    pub fn content_type(&self) -> &str {
        &self.content_type
    }

    /// Returns the exact `Content-Disposition` header value when it was valid text.
    pub fn content_disposition(&self) -> Option<&str> {
        self.content_disposition.as_deref()
    }
}

impl fmt::Debug for BinaryResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let media_type = self
            .content_type
            .split_once(';')
            .map_or(self.content_type.as_ref(), |(media_type, _)| media_type)
            .trim();
        formatter
            .debug_struct("BinaryResponse")
            .field("body_bytes", &self.bytes.len())
            .field("media_type", &media_type)
            .field(
                "has_content_disposition",
                &self.content_disposition.is_some(),
            )
            .finish()
    }
}

/// Backward-compatible name for an owned binary response.
pub type BinaryBody = BinaryResponse;

fn decode_binary(
    body: &[u8],
    metadata: ResponseMetadata<'_>,
) -> std::result::Result<BinaryResponse, ()> {
    Ok(BinaryResponse {
        bytes: body.to_vec(),
        content_type: metadata.content_type.into(),
        content_disposition: metadata.content_disposition.map(Into::into),
    })
}
