//! Stable, transport-independent error and diagnostic safety contracts.

use std::{borrow::Cow, collections::BTreeSet, error::Error as StdError, fmt};

use crate::types::{EmptyTickerList, InvalidDateRange, InvalidTemporalValue, StringValueError};

/// Replacement shown anywhere a secret value was removed.
pub const REDACTED: &str = "[REDACTED]";

/// Maximum UTF-8 byte length retained from a provider response body.
pub const MAX_SAFE_BODY_BYTES: usize = 4096;

/// Why a custom secret header or query name was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretNameError {
    /// Names must contain at least one character.
    Empty,
    /// Names accept only conservative ASCII token characters.
    InvalidCharacter,
}

impl fmt::Display for SecretNameError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("secret name must not be empty"),
            Self::InvalidCharacter => {
                formatter.write_str("secret name contains unsupported characters")
            }
        }
    }
}

impl StdError for SecretNameError {}

/// Broad, stable categories shared by Rust and Python callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorCategory {
    /// An input failed local validation.
    Validation,
    /// Client or transport configuration is invalid.
    Configuration,
    /// A request could not be completed by the transport.
    Transport,
    /// The provider returned a non-success HTTP status.
    Status,
    /// A successful response could not be decoded.
    Decode,
}

impl ErrorCategory {
    /// Returns the stable lowercase value exposed to language bindings.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::Configuration => "configuration",
            Self::Transport => "transport",
            Self::Status => "status",
            Self::Decode => "decode",
        }
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A secret whose formatted representations never reveal its value.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretString(Box<str>);

impl SecretString {
    /// Wraps a value that must not appear in diagnostics.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into().into_boxed_str())
    }

    /// Explicitly exposes the value for request construction.
    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(REDACTED)
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretString([REDACTED])")
    }
}

/// A signed, authenticated, or report URL that is wholly secret in diagnostics.
#[derive(Clone, PartialEq, Eq)]
pub struct SecretUrl(SecretString);

impl SecretUrl {
    /// Wraps a URL whose path or query may grant access to protected data.
    pub fn new(value: impl Into<String>) -> Self {
        Self(SecretString::new(value))
    }

    /// Explicitly exposes the URL for request construction.
    pub fn expose_secret(&self) -> &str {
        self.0.expose_secret()
    }
}

impl fmt::Display for SecretUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[REDACTED URL]")
    }
}

impl fmt::Debug for SecretUrl {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretUrl([REDACTED URL])")
    }
}

/// Reusable redaction policy for diagnostic text and provider bodies.
#[derive(Clone)]
pub struct Redactor {
    secrets: Vec<SecretString>,
    secret_query_names: BTreeSet<String>,
    secret_header_names: BTreeSet<String>,
}

impl Default for Redactor {
    fn default() -> Self {
        let secret_query_names = [
            "access_token",
            "api_key",
            "apikey",
            "key",
            "sig",
            "signature",
            "token",
            "x-amz-signature",
            "x-goog-signature",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        let secret_header_names = [
            "authorization",
            "proxy-authorization",
            "x-api-key",
            "apikey",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();

        Self {
            secrets: Vec::new(),
            secret_query_names,
            secret_header_names,
        }
    }
}

impl Redactor {
    /// Creates a policy with common authentication and signing names protected.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers another secret value to remove from arbitrary diagnostic text.
    pub fn add_secret(&mut self, secret: &SecretString) {
        if !secret.expose_secret().is_empty() {
            self.secrets.push(secret.clone());
        }
    }

    /// Marks an additional header name as secret, compared case-insensitively.
    pub fn add_secret_header_name(
        &mut self,
        name: impl Into<String>,
    ) -> std::result::Result<(), SecretNameError> {
        let name = validate_secret_header_name(name.into())?;
        self.secret_header_names.insert(name.to_ascii_lowercase());
        Ok(())
    }

    /// Marks an additional query name as secret, compared case-insensitively.
    pub fn add_secret_query_name(
        &mut self,
        name: impl Into<String>,
    ) -> std::result::Result<(), SecretNameError> {
        let name = validate_secret_query_name(name.into())?;
        self.secret_query_names.insert(name.to_ascii_lowercase());
        Ok(())
    }

    /// Produces a safe header value for diagnostics.
    pub fn redact_header(&self, name: &str, value: &str) -> String {
        if self
            .secret_header_names
            .contains(&name.to_ascii_lowercase())
        {
            REDACTED.to_owned()
        } else {
            self.redact(value)
        }
    }

    /// Removes registered values and values of known secret query parameters.
    pub fn redact(&self, value: &str) -> String {
        let mut redacted = value.to_owned();
        for secret in &self.secrets {
            redacted = redacted.replace(secret.expose_secret(), REDACTED);
        }
        redact_query_values(&redacted, &self.secret_query_names)
    }
}

fn validate_secret_header_name(name: String) -> std::result::Result<String, SecretNameError> {
    if name.is_empty() {
        return Err(SecretNameError::Empty);
    }
    if !name.bytes().all(|byte| {
        byte.is_ascii_alphanumeric()
            || matches!(
                byte,
                b'!' | b'#'
                    | b'$'
                    | b'%'
                    | b'&'
                    | b'\''
                    | b'*'
                    | b'+'
                    | b'-'
                    | b'.'
                    | b'^'
                    | b'_'
                    | b'`'
                    | b'|'
                    | b'~'
            )
    }) {
        return Err(SecretNameError::InvalidCharacter);
    }
    Ok(name)
}

fn validate_secret_query_name(name: String) -> std::result::Result<String, SecretNameError> {
    if name.is_empty() {
        return Err(SecretNameError::Empty);
    }
    if !name
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~'))
    {
        return Err(SecretNameError::InvalidCharacter);
    }
    Ok(name)
}

impl fmt::Debug for Redactor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Redactor")
            .field("registered_secret_count", &self.secrets.len())
            .field("secret_query_names", &self.secret_query_names)
            .field("secret_header_names", &self.secret_header_names)
            .finish()
    }
}

fn redact_query_values(value: &str, names: &BTreeSet<String>) -> String {
    let lowercase = value.to_ascii_lowercase();
    let mut ranges = Vec::new();

    for name in names {
        let needle = format!("{name}=");
        for (start, _) in lowercase.match_indices(&needle) {
            let valid_prefix = start == 0
                || matches!(
                    value.as_bytes()[start - 1],
                    b'?' | b'&' | b';' | b' ' | b'\'' | b'"'
                );
            if !valid_prefix {
                continue;
            }
            let value_start = start + needle.len();
            let value_end = value[value_start..]
                .find(|character: char| {
                    matches!(
                        character,
                        '&' | ';' | '#' | ' ' | '\n' | '\r' | '\t' | '\'' | '"'
                    )
                })
                .map_or(value.len(), |offset| value_start + offset);
            ranges.push((value_start, value_end));
        }
    }

    ranges.sort_unstable();
    let mut disjoint_ranges: Vec<(usize, usize)> = Vec::with_capacity(ranges.len());
    for (start, end) in ranges {
        if let Some((_, previous_end)) = disjoint_ranges.last_mut()
            && start <= *previous_end
        {
            *previous_end = (*previous_end).max(end);
        } else {
            disjoint_ranges.push((start, end));
        }
    }
    let mut output = value.to_owned();
    for (start, end) in disjoint_ranges.into_iter().rev() {
        output.replace_range(start..end, REDACTED);
    }
    output
}

/// A redacted, bounded provider response body suitable for diagnostics.
#[derive(Clone, PartialEq, Eq)]
pub struct SafeBody {
    text: Box<str>,
    truncated: bool,
}

impl SafeBody {
    /// Redacts and bounds a response body before retaining it in an error.
    pub fn new(body: &str, redactor: &Redactor) -> Self {
        let mut text = redactor.redact(body);
        let truncated = text.len() > MAX_SAFE_BODY_BYTES;
        if truncated {
            let mut end = MAX_SAFE_BODY_BYTES.saturating_sub('…'.len_utf8());
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            text.truncate(end);
            text.push('…');
        }
        Self {
            text: text.into_boxed_str(),
            truncated,
        }
    }

    /// Returns the safe retained body text.
    pub fn as_str(&self) -> &str {
        &self.text
    }

    /// Reports whether provider content was discarded to enforce the bound.
    pub fn is_truncated(&self) -> bool {
        self.truncated
    }
}

impl fmt::Display for SafeBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl fmt::Debug for SafeBody {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SafeBody")
            .field("text", &self.text)
            .field("truncated", &self.truncated)
            .finish()
    }
}

/// The stable error value returned by the Rust client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    category: ErrorCategory,
    message: Cow<'static, str>,
    endpoint: Option<&'static str>,
    status: Option<u16>,
    body: Option<SafeBody>,
}

impl Error {
    /// Creates a local input-validation error from a static, secret-free message.
    pub fn validation(message: &'static str) -> Self {
        Self::new(ErrorCategory::Validation, message)
    }

    /// Creates a configuration error from a static, secret-free message.
    pub fn configuration(message: &'static str) -> Self {
        Self::new(ErrorCategory::Configuration, message)
    }

    /// Creates a transport failure tied to an optional logical endpoint descriptor.
    ///
    /// `endpoint` is a static identity such as `"quote-short"`, never a URL.
    pub fn transport(endpoint: Option<&'static str>, message: &'static str) -> Self {
        Self::new(ErrorCategory::Transport, message).with_endpoint(endpoint)
    }

    /// Creates a non-success HTTP status error.
    /// `endpoint` is a static logical identity or relative path, never a URL.
    pub fn status(endpoint: &'static str, status: u16, body: Option<SafeBody>) -> Self {
        let mut error = Self::new(
            ErrorCategory::Status,
            format!("provider returned HTTP status {status}"),
        )
        .with_endpoint(Some(endpoint));
        error.status = Some(status);
        error.body = body;
        error
    }

    /// Creates a response decoding error with available response context.
    ///
    /// `endpoint` is a static logical identity or relative path, never a URL,
    /// and `message` is a static, secret-free descriptor.
    pub fn decode(
        endpoint: Option<&'static str>,
        status: Option<u16>,
        body: Option<SafeBody>,
        message: &'static str,
    ) -> Self {
        let mut error = Self::new(ErrorCategory::Decode, message).with_endpoint(endpoint);
        error.status = status;
        error.body = body;
        error
    }

    fn new(category: ErrorCategory, message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            category,
            message: message.into(),
            endpoint: None,
            status: None,
            body: None,
        }
    }

    fn with_endpoint(mut self, endpoint: Option<&'static str>) -> Self {
        self.endpoint = endpoint;
        self
    }

    /// Returns this error's stable category.
    pub fn category(&self) -> ErrorCategory {
        self.category
    }

    /// Returns the detail message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the logical endpoint identity or relative path, when relevant.
    pub fn endpoint(&self) -> Option<&str> {
        self.endpoint
    }

    /// Returns the provider HTTP status, when available.
    pub fn status_code(&self) -> Option<u16> {
        self.status
    }

    /// Returns the redacted, bounded provider body, when retained.
    pub fn body(&self) -> Option<&SafeBody> {
        self.body.as_ref()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)?;
        if let Some(endpoint) = &self.endpoint {
            write!(formatter, " (endpoint: {endpoint})")?;
        }
        if let Some(body) = &self.body {
            write!(formatter, ": {body}")?;
        }
        Ok(())
    }
}

impl StdError for Error {}

impl From<StringValueError> for Error {
    fn from(error: StringValueError) -> Self {
        let message = match error {
            StringValueError::Empty => "value must not be empty or whitespace-only",
            StringValueError::ControlCharacter => "value must not contain control characters",
            StringValueError::Comma => "ticker must not contain a comma",
        };
        Self::validation(message)
    }
}

impl From<EmptyTickerList> for Error {
    fn from(_: EmptyTickerList) -> Self {
        Self::validation("ticker list must contain at least one ticker")
    }
}

impl From<InvalidTemporalValue> for Error {
    fn from(_: InvalidTemporalValue) -> Self {
        Self::validation("invalid date or API datetime")
    }
}

impl From<InvalidDateRange> for Error {
    fn from(_: InvalidDateRange) -> Self {
        Self::validation("date range start must not be later than its end")
    }
}

/// Result alias used by client operations.
pub type Result<T> = std::result::Result<T, Error>;
