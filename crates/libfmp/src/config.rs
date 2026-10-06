//! Authentication and shared HTTP client configuration values.

use std::fmt;

use crate::error::SecretString;

/// The default Financial Modeling Prep API origin.
pub const DEFAULT_BASE_URL: &str = "https://financialmodelingprep.com";

/// The default path prefix used by the stable FMP API.
pub const DEFAULT_PATH_PREFIX: &str = "stable";

/// The process environment variable that [`fmp_api_key_from_env`] reads.
pub const FMP_API_KEY_ENV: &str = "FMP_API_KEY";

/// Reads the FMP API key from the `FMP_API_KEY` process environment variable.
///
/// Surrounding whitespace is trimmed; an unset, empty, or whitespace-only
/// value yields `None`. Callers that want the key applied as FMP header
/// authentication can use [`Authentication::fmp_header_from_env`] instead.
///
/// ```
/// use libfmp::config::{FMP_API_KEY_ENV, fmp_api_key_from_env};
///
/// let expected = std::env::var(FMP_API_KEY_ENV)
///     .ok()
///     .map(|value| value.trim().to_owned())
///     .filter(|value| !value.is_empty());
/// assert_eq!(fmp_api_key_from_env(), expected);
/// ```
pub fn fmp_api_key_from_env() -> Option<String> {
    fmp_api_key_from_value(std::env::var(FMP_API_KEY_ENV).ok().as_deref())
}

/// The process environment variable that [`fmp_base_url_from_env`] reads.
pub const FMP_BASE_URL_ENV: &str = "FMP_BASE_URL";

/// Reads a base URL override, such as a proxy origin, from the
/// `FMP_BASE_URL` process environment variable.
///
/// Surrounding whitespace is trimmed; an unset, empty, or whitespace-only
/// value yields `None`. The value is not validated here:
/// [`crate::ClientBuilder::build`] rejects anything that is not an absolute
/// HTTP(S) URL. [`crate::ClientBuilder::from_env`] applies it together with
/// the key.
///
/// ```
/// use libfmp::config::{FMP_BASE_URL_ENV, fmp_base_url_from_env};
///
/// let expected = std::env::var(FMP_BASE_URL_ENV)
///     .ok()
///     .map(|value| value.trim().to_owned())
///     .filter(|value| !value.is_empty());
/// assert_eq!(fmp_base_url_from_env(), expected);
/// ```
pub fn fmp_base_url_from_env() -> Option<String> {
    fmp_base_url_from_value(std::env::var(FMP_BASE_URL_ENV).ok().as_deref())
}

fn fmp_api_key_from_value(value: Option<&str>) -> Option<String> {
    trimmed_non_empty(value)
}

fn fmp_base_url_from_value(value: Option<&str>) -> Option<String> {
    trimmed_non_empty(value)
}

fn trimmed_non_empty(value: Option<&str>) -> Option<String> {
    let trimmed = value?.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

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

    /// Creates FMP's exact `apikey` header authentication from the
    /// `FMP_API_KEY` process environment variable.
    ///
    /// Returns `None` when the variable is unset, empty, or whitespace-only;
    /// see [`fmp_api_key_from_env`] for the exact normalization. The
    /// [`crate::ClientBuilder`] never reads the environment on its own, so
    /// Rust callers opt in explicitly:
    ///
    /// ```no_run
    /// use libfmp::{Client, config::Authentication};
    ///
    /// # fn main() -> libfmp::Result<()> {
    /// let auth = Authentication::fmp_header_from_env()
    ///     .ok_or_else(|| libfmp::Error::configuration("FMP_API_KEY is not set"))?;
    /// let client = Client::builder().authentication(auth).build()?;
    /// # let _ = client;
    /// # Ok(())
    /// # }
    /// ```
    pub fn fmp_header_from_env() -> Option<Self> {
        fmp_api_key_from_env().map(Self::fmp_header)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn env_key_is_absent_when_unset_empty_or_blank() {
        assert_eq!(fmp_api_key_from_value(None), None);
        assert_eq!(fmp_api_key_from_value(Some("")), None);
        assert_eq!(fmp_api_key_from_value(Some("   \t\n")), None);
    }

    #[test]
    fn env_key_is_trimmed_when_set() {
        assert_eq!(
            fmp_api_key_from_value(Some("shell-secret")).as_deref(),
            Some("shell-secret")
        );
        assert_eq!(
            fmp_api_key_from_value(Some("  shell-secret\n")).as_deref(),
            Some("shell-secret")
        );
    }

    #[test]
    fn env_helpers_agree_with_the_process_environment() {
        let expected = fmp_api_key_from_value(std::env::var(FMP_API_KEY_ENV).ok().as_deref());
        assert_eq!(fmp_api_key_from_env(), expected);
        assert_eq!(
            Authentication::fmp_header_from_env(),
            expected.map(Authentication::fmp_header)
        );
        assert_eq!(
            fmp_base_url_from_env(),
            fmp_base_url_from_value(std::env::var(FMP_BASE_URL_ENV).ok().as_deref())
        );
    }

    #[test]
    fn env_base_url_is_trimmed_and_absent_when_blank() {
        assert_eq!(fmp_base_url_from_value(None), None);
        assert_eq!(fmp_base_url_from_value(Some("")), None);
        assert_eq!(fmp_base_url_from_value(Some(" \t\n")), None);
        assert_eq!(
            fmp_base_url_from_value(Some("  https://valet.bitbrew.app/fmp\n")).as_deref(),
            Some("https://valet.bitbrew.app/fmp")
        );
    }
}
