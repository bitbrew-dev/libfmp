//! Authentication and shared HTTP client configuration values.

use std::fmt;

use crate::error::SecretString;

/// The default Financial Modeling Prep API origin.
pub const DEFAULT_BASE_URL: &str = "https://financialmodelingprep.com";

/// The default path prefix used by the stable FMP API.
pub const DEFAULT_PATH_PREFIX: &str = "stable";

/// Redirect behavior for requests that may carry credentials.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum RedirectPolicy {
    /// Return redirect responses without following them.
    None,
    /// Follow at most ten redirects when every hop has the same origin.
    #[default]
    SameOrigin,
}

/// Authentication applied by the transport to every request.
#[derive(Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum Authentication {
    /// Do not add an authentication header or query parameter.
    #[default]
    None,
    /// Send the FMP API key in the exact `apikey` header.
    FmpHeader(SecretString),
    /// Send the FMP API key in the `apikey` query parameter.
    FmpQuery(SecretString),
    /// Send an RFC 6750-style `Authorization: Bearer ...` header.
    Bearer(SecretString),
    /// Send a secret in a caller-selected header.
    CustomHeader {
        /// Header field name.
        name: String,
        /// Optional non-secret text placed immediately before the secret.
        prefix: Option<String>,
        /// Protected header value.
        secret: SecretString,
    },
    /// Send a secret in a caller-selected query parameter.
    CustomQuery {
        /// Query parameter name.
        name: String,
        /// Protected query value.
        secret: SecretString,
    },
}

impl Authentication {
    /// Creates FMP's exact `apikey` header authentication.
    pub fn fmp_header(api_key: impl Into<String>) -> Self {
        Self::FmpHeader(SecretString::new(api_key))
    }

    /// Creates FMP's `apikey` query authentication.
    pub fn fmp_query(api_key: impl Into<String>) -> Self {
        Self::FmpQuery(SecretString::new(api_key))
    }

    /// Creates bearer-token authentication.
    pub fn bearer(token: impl Into<String>) -> Self {
        Self::Bearer(SecretString::new(token))
    }

    /// Creates custom secret-header authentication.
    pub fn custom_header(
        name: impl Into<String>,
        prefix: Option<String>,
        secret: impl Into<String>,
    ) -> Self {
        Self::CustomHeader {
            name: name.into(),
            prefix,
            secret: SecretString::new(secret),
        }
    }

    /// Creates custom secret-query authentication.
    pub fn custom_query(name: impl Into<String>, secret: impl Into<String>) -> Self {
        Self::CustomQuery {
            name: name.into(),
            secret: SecretString::new(secret),
        }
    }
}

impl fmt::Debug for Authentication {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => formatter.write_str("None"),
            Self::FmpHeader(_) => formatter.write_str("FmpHeader([REDACTED])"),
            Self::FmpQuery(_) => formatter.write_str("FmpQuery([REDACTED])"),
            Self::Bearer(_) => formatter.write_str("Bearer([REDACTED])"),
            Self::CustomHeader { name, prefix, .. } => formatter
                .debug_struct("CustomHeader")
                .field("name", name)
                .field("has_prefix", &prefix.is_some())
                .field("secret", &"[REDACTED]")
                .finish(),
            Self::CustomQuery { name, .. } => formatter
                .debug_struct("CustomQuery")
                .field("name", name)
                .field("secret", &"[REDACTED]")
                .finish(),
        }
    }
}
