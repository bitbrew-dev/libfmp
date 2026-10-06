//! Shared async client and proxy-ready request transport.

use std::{
    collections::BTreeSet,
    fmt,
    future::{Future, poll_fn},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError, Sender},
    },
    task::{Context, Poll, Waker},
    thread,
    time::Duration,
};

use http::header::{
    CONTENT_DISPOSITION, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, LOCATION, USER_AGENT,
};
use url::Url;

use crate::{
    Result, VERSION,
    config::{
        Authentication, DEFAULT_BASE_URL, DEFAULT_PATH_PREFIX, RedirectPolicy,
        fmp_api_key_from_env, fmp_base_url_from_env,
    },
    endpoints::{
        DecodeFailure, ExpectedContentType, QueryEncoder, ResponseMetadata, is_provider_message,
    },
    error::{ConfigurationErrorKind, Error, Redactor, SafeBody, SafeHeaders, SecretString},
    transport::{HttpExecutor, PreparedRequest, ReqwestExecutor, TransportResponse},
};

pub use crate::endpoints::{EndpointSpec, QueryParameters};

/// Default logical deadline for one call, redirects and body included (30 s).
///
/// It applies only while the client timeout is unset;
/// [`ClientBuilder::timeout`] replaces it for every endpoint.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// Default logical deadline for one CSV bulk call (600 s).
///
/// Whole-market bulk bodies arrive slowly: `profile-bulk` part 0 measured
/// 31.4 MB in 202 s on 2026-10-02. It applies only while the client timeout
/// is unset; [`ClientBuilder::timeout`] replaces it for every endpoint, bulk
/// included (ADR 0035).
pub const DEFAULT_BULK_TIMEOUT: Duration = Duration::from_secs(600);
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
/// Default maximum bytes buffered for each HTTP response (64 MiB).
///
/// This finite default also applies to XLSX convenience methods. Callers
/// expecting a larger response can raise the client limit explicitly.
pub const DEFAULT_MAX_RESPONSE_BODY_BYTES: usize = 64 * 1024 * 1024;

/// Default maximum bytes buffered for each CSV bulk response (256 MiB).
///
/// Whole-market bulk bodies exceed the general default: `ratios-ttm-bulk`
/// measured 65.6 MiB on 2026-10-01. It applies only while the client limit
/// is unset; [`ClientBuilder::max_response_body_bytes`] replaces it for
/// every endpoint, bulk included (ADR 0035).
pub const DEFAULT_BULK_MAX_RESPONSE_BODY_BYTES: usize = 256 * 1024 * 1024;
const MAX_REDIRECTS: usize = 10;

#[derive(Clone)]
struct AuthMaterial {
    header: Option<(HeaderName, HeaderValue)>,
    query: Option<(String, SecretString)>,
}

impl fmt::Debug for AuthMaterial {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthMaterial")
            .field("header_name", &self.header.as_ref().map(|(name, _)| name))
            .field("query_name", &self.query.as_ref().map(|(name, _)| name))
            .finish()
    }
}

/// Builder for the shared async `Client`.
#[derive(Clone)]
pub struct ClientBuilder {
    base_url: String,
    path_prefix: String,
    authentication: Authentication,
    authentication_configured: bool,
    authentication_conflict: bool,
    default_headers: Vec<(String, String)>,
    user_agent: String,
    timeout: Option<Duration>,
    connect_timeout: Duration,
    max_response_body_bytes: Option<usize>,
    danger_allow_insecure_authentication: bool,
    redirect_policy: RedirectPolicy,
    executor: Option<Arc<dyn HttpExecutor>>,
    on_response: Option<ResponseObserver>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_owned(),
            path_prefix: DEFAULT_PATH_PREFIX.to_owned(),
            authentication: Authentication::None,
            authentication_configured: false,
            authentication_conflict: false,
            default_headers: Vec::new(),
            user_agent: format!("libfmp/{VERSION}"),
            timeout: None,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            max_response_body_bytes: None,
            danger_allow_insecure_authentication: false,
            redirect_policy: RedirectPolicy::SameOrigin,
            executor: None,
            on_response: None,
        }
    }
}

impl fmt::Debug for ClientBuilder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let header_names: Vec<_> = self
            .default_headers
            .iter()
            .map(|(name, _)| name.as_str())
            .collect();
        formatter
            .debug_struct("ClientBuilder")
            .field("base_url", &"[CONFIGURED URL]")
            .field("path_prefix", &"[CONFIGURED PATH]")
            .field("authentication", &self.authentication)
            .field("authentication_conflict", &self.authentication_conflict)
            .field("default_header_names", &header_names)
            .field("user_agent", &"[CONFIGURED USER AGENT]")
            .field("timeout", &self.timeout)
            .field("connect_timeout", &self.connect_timeout)
            .field("max_response_body_bytes", &self.max_response_body_bytes)
            .field(
                "danger_allow_insecure_authentication",
                &self.danger_allow_insecure_authentication,
            )
            .field("redirect_policy", &self.redirect_policy)
            .field("custom_executor", &self.executor.is_some())
            .field("response_observer", &self.on_response.is_some())
            .finish()
    }
}

impl ClientBuilder {
    /// Starts a builder from the `FMP_API_KEY` and `FMP_BASE_URL` process
    /// environment variables, for example to route through a proxy that
    /// issues its own keys.
    ///
    /// The key becomes [`Authentication::FmpHeader`] (the `apikey` header);
    /// the base URL, when set, replaces [`DEFAULT_BASE_URL`] and keeps its
    /// path, so `https://valet.bitbrew.app/fmp` sends the quote endpoint to
    /// `https://valet.bitbrew.app/fmp/stable/quote`. The path prefix stays
    /// [`DEFAULT_PATH_PREFIX`]. Both values are trimmed, and an empty or
    /// whitespace-only value counts as unset (see
    /// [`crate::config::fmp_api_key_from_env`] and
    /// [`crate::config::fmp_base_url_from_env`]).
    ///
    /// This is the only constructor that reads the environment:
    /// [`Client::builder`] never does. The returned builder already has its
    /// authentication selected, so calling [`Self::authentication`] on it
    /// fails at [`Self::build`] with
    /// [`ConfigurationErrorKind::ConflictingAuthentication`]; every other
    /// setter, including [`Self::base_url`], still applies.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigurationErrorKind::MissingCredential`] when
    /// `FMP_API_KEY` is unset or blank. An `FMP_BASE_URL` that is not an
    /// absolute HTTP(S) URL fails at [`Self::build`] with
    /// [`ConfigurationErrorKind::InvalidBaseUrl`] or
    /// [`ConfigurationErrorKind::UnsafeBaseUrl`], and a plaintext
    /// non-loopback `http://` URL with
    /// [`ConfigurationErrorKind::InsecureAuthentication`] unless
    /// [`Self::danger_allow_insecure_authentication`] is set.
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use libfmp::ClientBuilder;
    ///
    /// # fn main() -> libfmp::Result<()> {
    /// // FMP_API_KEY=vk_... FMP_BASE_URL=https://valet.bitbrew.app/fmp
    /// let client = ClientBuilder::from_env()?
    ///     .timeout(Duration::from_secs(10))
    ///     .build()?;
    /// # let _ = client;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_env() -> Result<Self> {
        Self::from_env_values(fmp_api_key_from_env(), fmp_base_url_from_env())
    }

    fn from_env_values(api_key: Option<String>, base_url: Option<String>) -> Result<Self> {
        let api_key = api_key.ok_or_else(|| {
            Error::configuration_with_kind(
                ConfigurationErrorKind::MissingCredential,
                "FMP_API_KEY is not set",
            )
        })?;
        let builder = Self::default().authentication(Authentication::fmp_header(api_key));
        Ok(match base_url {
            Some(base_url) => builder.base_url(base_url),
            None => builder,
        })
    }

    /// Sets an absolute HTTP(S) base URL. Existing base path segments are kept.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Sets path segments inserted between the base URL and endpoint path.
    ///
    /// An empty prefix is valid for proxies that expose endpoints at their root;
    /// optional leading and trailing slashes are ignored without resetting the
    /// base URL's existing path.
    pub fn path_prefix(mut self, path_prefix: impl Into<String>) -> Self {
        self.path_prefix = path_prefix.into();
        self
    }

    /// Selects one transport-owned authentication mode.
    pub fn authentication(mut self, authentication: Authentication) -> Self {
        if self.authentication_configured {
            self.authentication_conflict = true;
        }
        self.authentication_configured = true;
        self.authentication = authentication;
        self
    }

    /// Adds or replaces a default header. Later calls win case-insensitively.
    pub fn default_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.push((name.into(), value.into()));
        self
    }

    /// Sets the transport-owned `User-Agent` header.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Sets one logical transport timeout across redirects and body buffering.
    ///
    /// The client enforces this deadline around custom executors as well as the
    /// built-in executor; individual redirect hops do not reset it. Until this
    /// is set, each endpoint uses its own default: [`DEFAULT_TIMEOUT`], or
    /// [`DEFAULT_BULK_TIMEOUT`] for CSV bulk routes. Once set, it applies to
    /// every endpoint, bulk included.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Sets the connection establishment timeout.
    pub fn connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.connect_timeout = connect_timeout;
        self
    }

    /// Sets the largest body buffered for any one response, in bytes.
    ///
    /// The built-in executor skips redirect bodies and bounds every success or
    /// error body while streaming. Buffers returned by custom executors are
    /// checked defensively for every status, including redirects. An endpoint
    /// can replace the limit with
    /// [`EndpointSpec::with_max_response_body_bytes`]. Until this is set,
    /// each endpoint uses its own default: [`DEFAULT_MAX_RESPONSE_BODY_BYTES`],
    /// or [`DEFAULT_BULK_MAX_RESPONSE_BODY_BYTES`] for CSV bulk routes. Once
    /// set, it applies to every endpoint, bulk and XLSX included.
    pub fn max_response_body_bytes(mut self, max_bytes: usize) -> Self {
        self.max_response_body_bytes = Some(max_bytes);
        self
    }

    /// Allows credentials to be sent over plaintext HTTP to a non-loopback host.
    ///
    /// This opt-in is intentionally conspicuous: plaintext transport can expose
    /// API keys and bearer tokens to intermediaries. It is unnecessary for
    /// literal IPv4 and IPv6 loopback URLs used by local test servers.
    pub fn danger_allow_insecure_authentication(mut self, allow: bool) -> Self {
        self.danger_allow_insecure_authentication = allow;
        self
    }

    /// Selects whether redirects are disabled or restricted to the same origin.
    pub fn redirect_policy(mut self, redirect_policy: RedirectPolicy) -> Self {
        self.redirect_policy = redirect_policy;
        self
    }

    /// Injects an executor, primarily for deterministic tests and adapters.
    pub fn executor(mut self, executor: Arc<dyn HttpExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Registers an observer called after every successful response.
    ///
    /// It receives a [`ResponseInfo`] with the endpoint id, the HTTP status
    /// and the same allowlisted, redacted [`SafeHeaders`] that
    /// [`Client::execute_with_metadata`] returns, for every call made through
    /// the client, endpoint methods included. It fires once per call, only
    /// after the body decoded, never on status, provider-message, decode or
    /// transport errors (those carry their headers on the [`Error`]). It
    /// returns `()`, so it cannot fail the call.
    ///
    /// The observer runs inline on the task driving the request, before the
    /// call returns: keep it cheap and non-blocking (record a value, send on a
    /// channel). A later call replaces an earlier observer.
    ///
    /// ```
    /// use std::sync::{Arc, Mutex};
    ///
    /// let remaining = Arc::new(Mutex::new(None::<String>));
    /// let seen = Arc::clone(&remaining);
    /// let builder = libfmp::Client::builder().on_response(move |info| {
    ///     let value = info.headers.get("x-proxy-daily-remaining").map(str::to_owned);
    ///     *seen.lock().unwrap() = value;
    /// });
    /// # let _ = builder;
    /// ```
    pub fn on_response(
        mut self,
        observer: impl Fn(&ResponseInfo<'_>) + Send + Sync + 'static,
    ) -> Self {
        self.on_response = Some(Arc::new(observer));
        self
    }

    /// Validates configuration and builds a cheap-to-clone async client.
    pub fn build(self) -> Result<Client> {
        if self.authentication_conflict {
            return Err(Error::configuration_with_kind(
                ConfigurationErrorKind::ConflictingAuthentication,
                "authentication can be configured only once",
            ));
        }
        let base_url = parse_base_url(&self.base_url)?;
        validate_relative_path(&self.path_prefix)?;
        let auth = build_auth_material(&self.authentication)?;
        if !matches!(self.authentication, Authentication::None)
            && base_url.scheme() == "http"
            && !is_loopback_origin(&base_url)
            && !self.danger_allow_insecure_authentication
        {
            return Err(Error::configuration_with_kind(
                ConfigurationErrorKind::InsecureAuthentication,
                "authenticated plaintext HTTP requires an explicit dangerous opt-in",
            ));
        }

        let mut protected_headers = reserved_header_names();
        if let Some((name, _)) = &auth.header {
            protected_headers.insert(name.as_str().to_ascii_lowercase());
        }

        let mut default_headers = parse_headers(&self.default_headers, &protected_headers)?;
        let user_agent = parse_header_value(&self.user_agent)?;
        default_headers.insert(USER_AGENT, user_agent);
        if matches!(self.authentication, Authentication::None) && is_default_fmp_origin(&base_url) {
            return Err(Error::configuration_with_kind(
                ConfigurationErrorKind::MissingCredential,
                "direct FMP access requires explicit authentication",
            ));
        }

        let mut redactor = Redactor::new();
        register_auth_redaction(&mut redactor, &self.authentication)?;
        redactor.add_secret(&SecretString::new(self.base_url.clone()));
        redactor.add_secret(&SecretString::new(base_url.as_str().to_owned()));
        redactor.add_secret(&SecretString::new(self.path_prefix.clone()));
        redactor.add_secret(&SecretString::new(self.user_agent.clone()));
        for (_, value) in &self.default_headers {
            redactor.add_secret(&SecretString::new(value.clone()));
        }

        let (executor, deadline_backend): (Arc<dyn HttpExecutor>, DeadlineBackend) =
            match self.executor {
                Some(executor) => (executor, DeadlineBackend::Thread),
                None => (
                    Arc::new(ReqwestExecutor::new(self.connect_timeout).map_err(|_| {
                        Error::configuration_with_kind(
                            ConfigurationErrorKind::HttpClient,
                            "HTTP client could not be constructed",
                        )
                    })?),
                    DeadlineBackend::Tokio,
                ),
            };

        Ok(Client {
            inner: Arc::new(ClientInner {
                base_url,
                path_prefix: self.path_prefix,
                auth,
                default_headers,
                protected_headers,
                redactor,
                executor,
                deadline_backend,
                timeout: self.timeout,
                max_response_body_bytes: self.max_response_body_bytes,
                redirect_policy: self.redirect_policy,
                on_response: self.on_response,
            }),
        })
    }
}

struct ClientInner {
    base_url: Url,
    path_prefix: String,
    auth: AuthMaterial,
    default_headers: HeaderMap,
    protected_headers: BTreeSet<String>,
    redactor: Redactor,
    executor: Arc<dyn HttpExecutor>,
    deadline_backend: DeadlineBackend,
    timeout: Option<Duration>,
    max_response_body_bytes: Option<usize>,
    redirect_policy: RedirectPolicy,
    on_response: Option<ResponseObserver>,
}

/// Async Financial Modeling Prep client shared by all endpoint modules.
#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

/// The observer registered with [`ClientBuilder::on_response`].
pub type ResponseObserver = Arc<dyn Fn(&ResponseInfo<'_>) + Send + Sync>;

/// What a [`ResponseObserver`] learns about one successful response.
///
/// It never carries the body or any header outside the allowlist.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct ResponseInfo<'a> {
    /// The registry id of the endpoint, for example `"quote"`.
    pub endpoint: &'static str,
    /// The HTTP status of the final (post-redirect) response.
    pub status: u16,
    /// The allowlisted, redacted headers of the final response.
    pub headers: &'a SafeHeaders,
}

/// A decoded success with the HTTP status and allowlisted response headers.
///
/// Returned by [`Client::execute_with_metadata`]. `headers` holds only the
/// names accepted by [`crate::error::is_retained_header_name`], redacted and
/// bounded exactly like [`Error::headers`].
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct Response<T> {
    /// The decoded endpoint data, as [`Client::execute`] returns it.
    pub data: T,
    /// The HTTP status of the final (post-redirect) response.
    pub status: u16,
    /// The allowlisted, redacted headers of the final response.
    pub headers: SafeHeaders,
}

impl fmt::Debug for Client {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Client")
            .field("base_url", &"[CONFIGURED URL]")
            .field("path_prefix", &"[CONFIGURED PATH]")
            .field("authentication", &self.inner.auth)
            .field("redirect_policy", &self.inner.redirect_policy)
            .field(
                "default_header_names",
                &self
                    .inner
                    .default_headers
                    .keys()
                    .map(HeaderName::as_str)
                    .collect::<Vec<_>>(),
            )
            .finish_non_exhaustive()
    }
}

impl Client {
    /// Starts a client builder with the direct FMP stable API defaults.
    ///
    /// The builder never reads the environment; see [`Self::from_env`].
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Builds a client from `FMP_API_KEY` and `FMP_BASE_URL`.
    ///
    /// Shorthand for `ClientBuilder::from_env()?.build()`; see
    /// [`ClientBuilder::from_env`] for the rules and errors.
    pub fn from_env() -> Result<Self> {
        ClientBuilder::from_env()?.build()
    }

    /// Executes a typed endpoint using only transport-owned default headers.
    pub async fn execute<Q, R>(&self, endpoint: &EndpointSpec<Q, R>) -> Result<R>
    where
        Q: QueryParameters,
    {
        self.execute_with_metadata(endpoint)
            .await
            .map(|response| response.data)
    }

    /// Executes a typed endpoint and keeps the status and allowlisted headers.
    ///
    /// The request, redirects and decoding are exactly those of
    /// [`Self::execute`]; on success the decoded data comes back with the HTTP
    /// status and the [`SafeHeaders`] of the final response, filtered by the
    /// same allowlist as [`Error::headers`] (`retry-after`, `x-proxy-*`,
    /// `x-ratelimit-*`) and redacted. Errors are unchanged and already carry
    /// their own headers.
    ///
    /// Endpoint spec functions are public, so any endpoint method has a
    /// metadata form. Reading a proxy's cache status and remaining budget:
    ///
    /// ```no_run
    /// use libfmp::{Client, endpoints::quote::{QuoteQuery, quote}, types::Ticker};
    ///
    /// # async fn run() -> libfmp::Result<()> {
    /// let client = Client::from_env()?;
    /// let spec = quote(QuoteQuery::new(Ticker::new("AAPL")?));
    /// let response = client.execute_with_metadata(&spec).await?;
    /// let cache = response.headers.get("x-proxy-cache");
    /// let remaining = response.headers.get("x-proxy-daily-remaining");
    /// println!("{} rows, cache {cache:?}, remaining {remaining:?}", response.data.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_with_metadata<Q, R>(
        &self,
        endpoint: &EndpointSpec<Q, R>,
    ) -> Result<Response<R>>
    where
        Q: QueryParameters,
    {
        self.execute_with_headers(endpoint, &[]).await
    }

    /// Executes a typed endpoint with request-specific header overrides.
    ///
    /// Request headers override defaults, but transport-owned authentication
    /// and `User-Agent` headers cannot be replaced.
    pub(crate) async fn execute_with_headers<Q, R>(
        &self,
        endpoint: &EndpointSpec<Q, R>,
        request_headers: &[(&str, &str)],
    ) -> Result<Response<R>>
    where
        Q: QueryParameters,
    {
        validate_relative_path(endpoint.relative_path())?;
        let mut url = build_endpoint_url(
            &self.inner.base_url,
            &self.inner.path_prefix,
            endpoint.relative_path(),
        )?;
        append_endpoint_query(&mut url, endpoint.query(), self.inner.auth.query.as_ref())?;

        let mut headers = self.inner.default_headers.clone();
        merge_request_headers(&mut headers, request_headers, &self.inner.protected_headers)?;
        if let Some((name, value)) = &self.inner.auth.header {
            headers.insert(name.clone(), value.clone());
        }

        let mut redactor = self.inner.redactor.clone();
        for (_, value) in request_headers {
            redactor.add_secret(&SecretString::new((*value).to_owned()));
        }

        let timeout = self.inner.timeout.unwrap_or(endpoint.default_timeout());
        with_timeout(
            self.inner.deadline_backend,
            timeout,
            self.execute_redirects(endpoint, url, headers, &redactor, timeout),
        )
        .await
        .ok_or_else(|| Error::transport(Some(endpoint.id()), "request deadline exceeded"))?
    }

    async fn execute_redirects<Q, R>(
        &self,
        endpoint: &EndpointSpec<Q, R>,
        mut url: Url,
        headers: HeaderMap,
        redactor: &Redactor,
        timeout: Duration,
    ) -> Result<Response<R>> {
        for redirect_count in 0..=MAX_REDIRECTS {
            apply_query_auth(&mut url, self.inner.auth.query.as_ref());
            let max_body_bytes = endpoint
                .max_response_body_bytes()
                .or(self.inner.max_response_body_bytes)
                .unwrap_or(endpoint.default_max_response_body_bytes());
            let response = self
                .inner
                .executor
                .execute(PreparedRequest::new(
                    endpoint.method(),
                    url.clone(),
                    headers.clone(),
                    max_body_bytes,
                    timeout,
                ))
                .await
                .map_err(|_| Error::transport(Some(endpoint.id()), "request execution failed"))?;
            if response.body_limit_exceeded() || response.body().len() > max_body_bytes {
                return Err(Error::transport(
                    Some(endpoint.id()),
                    "response body exceeded configured limit",
                ));
            }

            if is_redirect(response.status()) {
                if self.inner.redirect_policy == RedirectPolicy::None {
                    return status_error(endpoint.id(), response, redactor);
                }
                if redirect_count == MAX_REDIRECTS {
                    return Err(Error::transport(
                        Some(endpoint.id()),
                        "same-origin redirect limit exceeded",
                    ));
                }
                let Some(location) = response.headers().get(LOCATION) else {
                    return status_error(endpoint.id(), response, redactor);
                };
                let Ok(location) = location.to_str() else {
                    return status_error(endpoint.id(), response, redactor);
                };
                let Ok(mut destination) = url.join(location) else {
                    return status_error(endpoint.id(), response, redactor);
                };
                destination.set_fragment(None);
                if destination.username() != ""
                    || destination.password().is_some()
                    || !same_origin(&url, &destination)
                {
                    return status_error(endpoint.id(), response, redactor);
                }
                url = destination;
                continue;
            }

            if !(200..300).contains(&response.status()) {
                return status_error(endpoint.id(), response, redactor);
            }

            let expected = endpoint.response().expected_content_type();
            let Some(content_type) = response.headers().get(CONTENT_TYPE) else {
                return Err(Error::decode(
                    Some(endpoint.id()),
                    Some(response.status()),
                    Some(error_body(expected, response.body(), None, redactor)),
                    "successful response omitted its content type",
                ));
            };
            let Ok(content_type) = content_type.to_str() else {
                return Err(Error::decode(
                    Some(endpoint.id()),
                    Some(response.status()),
                    Some(error_body(expected, response.body(), None, redactor)),
                    "successful response used an invalid content type",
                ));
            };
            if !expected.matches(content_type) && !expected.matches_signature(response.body()) {
                if expected.is_binary()
                    && ExpectedContentType::Json.matches(content_type)
                    && is_provider_message(response.body())
                {
                    return Err(Error::provider_message(
                        endpoint.id(),
                        response.status(),
                        Some(safe_body(response.body(), redactor)),
                    )
                    .with_headers(safe_headers(&response, redactor)));
                }
                return Err(Error::decode(
                    Some(endpoint.id()),
                    Some(response.status()),
                    Some(error_body(
                        expected,
                        response.body(),
                        Some(content_type),
                        redactor,
                    )),
                    "successful response used an unexpected content type",
                ));
            }

            let content_disposition = response
                .headers()
                .get(CONTENT_DISPOSITION)
                .and_then(|value| value.to_str().ok());

            return endpoint
                .response()
                .decode(
                    response.body_bytes(),
                    ResponseMetadata::new(content_type, content_disposition),
                )
                .map(|data| {
                    let success = Response {
                        data,
                        status: response.status(),
                        headers: safe_headers(&response, redactor),
                    };
                    if let Some(observer) = &self.inner.on_response {
                        observer(&ResponseInfo {
                            endpoint: endpoint.id(),
                            status: success.status,
                            headers: &success.headers,
                        });
                    }
                    success
                })
                .map_err(|failure| match failure {
                    DecodeFailure::Shape { path, kind } => Error::decode(
                        Some(endpoint.id()),
                        Some(response.status()),
                        Some(safe_body(response.body(), redactor)),
                        "successful response could not be decoded",
                    )
                    .with_decode_location(path.map(|path| redactor.redact(&path)), kind),
                    DecodeFailure::ProviderMessage => Error::provider_message(
                        endpoint.id(),
                        response.status(),
                        Some(safe_body(response.body(), redactor)),
                    )
                    .with_headers(safe_headers(&response, redactor)),
                });
        }

        Err(Error::transport(
            Some(endpoint.id()),
            "redirect processing ended unexpectedly",
        ))
    }
}

async fn with_timeout<F>(
    backend: DeadlineBackend,
    duration: Duration,
    future: F,
) -> Option<F::Output>
where
    F: Future,
{
    match backend {
        DeadlineBackend::Tokio => tokio::time::timeout(duration, future).await.ok(),
        DeadlineBackend::Thread => with_thread_timeout(duration, future).await,
    }
}

async fn with_thread_timeout<F>(duration: Duration, future: F) -> Option<F::Output>
where
    F: Future,
{
    let mut future = Box::pin(future);
    let mut deadline = Box::pin(ThreadDeadline::new(duration));
    poll_fn(move |context| {
        if let Poll::Ready(output) = future.as_mut().poll(context) {
            return Poll::Ready(Some(output));
        }
        if deadline.as_mut().poll(context).is_ready() {
            Poll::Ready(None)
        } else {
            Poll::Pending
        }
    })
    .await
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeadlineBackend {
    Tokio,
    Thread,
}

/// Runtime-neutral deadline used for injected executors.
///
/// Each fallback deadline owns a cancellable thread rather than retaining a
/// process-global timer handle. Process-aware bindings can therefore rebuild
/// their runtime after `fork` without inheriting this fallback's timer state.
struct ThreadDeadline {
    state: Arc<ThreadDeadlineState>,
    cancel: Option<Sender<()>>,
    thread: Option<thread::JoinHandle<()>>,
}

struct ThreadDeadlineState {
    elapsed: AtomicBool,
    waker: Mutex<Option<Waker>>,
}

impl ThreadDeadline {
    fn new(duration: Duration) -> Self {
        let state = Arc::new(ThreadDeadlineState {
            elapsed: AtomicBool::new(false),
            waker: Mutex::new(None),
        });
        let thread_state = Arc::clone(&state);
        let (cancel, receiver) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("libfmp-deadline".to_owned())
            .spawn(move || {
                if matches!(
                    receiver.recv_timeout(duration),
                    Err(RecvTimeoutError::Timeout)
                ) {
                    thread_state.elapsed.store(true, Ordering::Release);
                    let waker = match thread_state.waker.lock() {
                        Ok(mut slot) => slot.take(),
                        Err(poisoned) => poisoned.into_inner().take(),
                    };
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                }
            });

        let (cancel, thread) = match spawned {
            Ok(thread) => (Some(cancel), Some(thread)),
            Err(_) => {
                state.elapsed.store(true, Ordering::Release);
                (None, None)
            }
        };

        Self {
            state,
            cancel,
            thread,
        }
    }
}

impl Future for ThreadDeadline {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.elapsed.load(Ordering::Acquire) {
            return Poll::Ready(());
        }

        match self.state.waker.lock() {
            Ok(mut slot) => {
                if slot
                    .as_ref()
                    .is_none_or(|registered| !registered.will_wake(context.waker()))
                {
                    *slot = Some(context.waker().clone());
                }
            }
            Err(mut poisoned) => {
                **poisoned.get_mut() = Some(context.waker().clone());
            }
        }

        if self.state.elapsed.load(Ordering::Acquire) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl Drop for ThreadDeadline {
    fn drop(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            let _ = cancel.send(());
        }
        if let Some(thread) = self.thread.take()
            && thread.thread().id() != thread::current().id()
        {
            let _ = thread.join();
        }
    }
}

fn parse_base_url(value: &str) -> Result<Url> {
    let url = Url::parse(value).map_err(|_| {
        Error::configuration_with_kind(
            ConfigurationErrorKind::InvalidBaseUrl,
            "base URL must be an absolute HTTP(S) URL",
        )
    })?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
        return Err(Error::configuration_with_kind(
            ConfigurationErrorKind::InvalidBaseUrl,
            "base URL must be an absolute HTTP(S) URL",
        ));
    }
    if url.username() != ""
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(Error::configuration_with_kind(
            ConfigurationErrorKind::UnsafeBaseUrl,
            "base URL must not contain credentials, a query, or a fragment",
        ));
    }
    Ok(url)
}

fn is_default_fmp_origin(url: &Url) -> bool {
    url.scheme() == "https"
        && url.host_str() == Some("financialmodelingprep.com")
        && url.port_or_known_default() == Some(443)
}

fn is_loopback_origin(url: &Url) -> bool {
    match url.host() {
        Some(url::Host::Ipv4(address)) => address.is_loopback(),
        Some(url::Host::Ipv6(address)) => address.is_loopback(),
        Some(url::Host::Domain(_)) | None => false,
    }
}

fn validate_relative_path(value: &str) -> Result<()> {
    if value.chars().any(char::is_control)
        || value.contains(['\\', '?', '#'])
        || value
            .trim_matches('/')
            .split('/')
            .any(|segment| matches!(segment, "." | ".."))
    {
        return Err(Error::configuration_with_kind(
            ConfigurationErrorKind::InvalidPath,
            "path must contain only safe relative segments",
        ));
    }
    Ok(())
}

fn build_endpoint_url(base: &Url, prefix: &str, relative_path: &str) -> Result<Url> {
    let mut url = base.clone();
    {
        let mut segments = url.path_segments_mut().map_err(|_| {
            Error::configuration_with_kind(
                ConfigurationErrorKind::InvalidBaseUrl,
                "base URL cannot contain path segments",
            )
        })?;
        segments.pop_if_empty();
        for segment in prefix
            .trim_matches('/')
            .split('/')
            .chain(relative_path.trim_matches('/').split('/'))
            .filter(|segment| !segment.is_empty())
        {
            segments.push(segment);
        }
    }
    Ok(url)
}

fn append_endpoint_query<Q: QueryParameters>(
    url: &mut Url,
    query: &Q,
    auth_query: Option<&(String, SecretString)>,
) -> Result<()> {
    let mut failure = None;
    query.encode(&mut QueryEncoder::new(&mut |name, value| {
        if failure.is_some() {
            return;
        }
        if name.is_empty() || name.chars().any(char::is_control) {
            failure = Some(Error::configuration_with_kind(
                ConfigurationErrorKind::InvalidQueryName,
                "query parameter name must not be empty or contain controls",
            ));
            return;
        }
        if auth_query.is_some_and(|(protected, _)| name.eq_ignore_ascii_case(protected)) {
            failure = Some(Error::configuration_with_kind(
                ConfigurationErrorKind::ProtectedFieldCollision,
                "endpoint query collides with transport authentication",
            ));
            return;
        }
        url.query_pairs_mut().append_pair(name, value);
    }));
    failure.map_or(Ok(()), Err)
}

fn apply_query_auth(url: &mut Url, auth_query: Option<&(String, SecretString)>) {
    let Some((protected_name, secret)) = auth_query else {
        return;
    };
    let retained: Vec<(String, String)> = url
        .query_pairs()
        .filter(|(name, _)| !name.eq_ignore_ascii_case(protected_name))
        .map(|(name, value)| (name.into_owned(), value.into_owned()))
        .collect();
    url.set_query(None);
    {
        let mut pairs = url.query_pairs_mut();
        pairs.extend_pairs(retained);
        pairs.append_pair(protected_name, secret.expose_secret());
    }
}

fn build_auth_material(authentication: &Authentication) -> Result<AuthMaterial> {
    match authentication {
        Authentication::None => Ok(AuthMaterial {
            header: None,
            query: None,
        }),
        Authentication::FmpHeader(secret) => Ok(AuthMaterial {
            header: Some(secret_header("apikey", "", secret)?),
            query: None,
        }),
        Authentication::FmpQuery(secret) => {
            validate_credential(secret)?;
            Ok(AuthMaterial {
                header: None,
                query: Some(("apikey".to_owned(), secret.clone())),
            })
        }
        Authentication::Bearer(secret) => Ok(AuthMaterial {
            header: Some(secret_header("authorization", "Bearer ", secret)?),
            query: None,
        }),
        Authentication::CustomHeader {
            name,
            prefix,
            secret,
        } => Ok(AuthMaterial {
            header: Some(secret_header(
                name,
                prefix.as_deref().unwrap_or(""),
                secret,
            )?),
            query: None,
        }),
        Authentication::CustomQuery { name, secret } => {
            validate_query_name(name)?;
            validate_credential(secret)?;
            Ok(AuthMaterial {
                header: None,
                query: Some((name.clone(), secret.clone())),
            })
        }
    }
}

fn secret_header(
    name: &str,
    prefix: &str,
    secret: &SecretString,
) -> Result<(HeaderName, HeaderValue)> {
    validate_credential(secret)?;
    let name = parse_header_name(name)?;
    if is_unsafe_transport_header(&name) {
        return Err(Error::configuration_with_kind(
            ConfigurationErrorKind::ProtectedFieldCollision,
            "authentication header is owned by HTTP framing or transport",
        ));
    }
    let mut value = parse_header_value(&format!("{prefix}{}", secret.expose_secret()))?;
    value.set_sensitive(true);
    Ok((name, value))
}

fn validate_credential(secret: &SecretString) -> Result<()> {
    if secret.expose_secret().is_empty() {
        Err(Error::configuration_with_kind(
            ConfigurationErrorKind::EmptyCredential,
            "authentication credential must not be empty",
        ))
    } else {
        Ok(())
    }
}

fn validate_query_name(name: &str) -> Result<()> {
    if name.is_empty() || name.chars().any(char::is_control) {
        Err(Error::configuration_with_kind(
            ConfigurationErrorKind::InvalidQueryName,
            "secret query name must not be empty or contain controls",
        ))
    } else {
        Ok(())
    }
}

fn parse_header_name(name: &str) -> Result<HeaderName> {
    HeaderName::from_bytes(name.as_bytes()).map_err(|_| {
        Error::configuration_with_kind(
            ConfigurationErrorKind::InvalidHeaderName,
            "header name is not a valid HTTP field name",
        )
    })
}

fn parse_header_value(value: &str) -> Result<HeaderValue> {
    HeaderValue::from_str(value).map_err(|_| {
        Error::configuration_with_kind(
            ConfigurationErrorKind::InvalidHeaderValue,
            "header value is not a valid HTTP field value",
        )
    })
}

fn parse_headers(headers: &[(String, String)], protected: &BTreeSet<String>) -> Result<HeaderMap> {
    let mut parsed = HeaderMap::new();
    for (name, value) in headers {
        let name = parse_header_name(name)?;
        if protected.contains(name.as_str()) {
            return Err(Error::configuration_with_kind(
                ConfigurationErrorKind::ProtectedFieldCollision,
                "default header collides with a transport-owned header",
            ));
        }
        let mut value = parse_header_value(value)?;
        if is_conventionally_secret_header(&name) {
            value.set_sensitive(true);
        }
        parsed.insert(name, value);
    }
    Ok(parsed)
}

fn merge_request_headers(
    headers: &mut HeaderMap,
    request_headers: &[(&str, &str)],
    protected: &BTreeSet<String>,
) -> Result<()> {
    for (name, value) in request_headers {
        let name = parse_header_name(name)?;
        if protected.contains(name.as_str()) {
            return Err(Error::configuration_with_kind(
                ConfigurationErrorKind::ProtectedFieldCollision,
                "request header collides with a transport-owned header",
            ));
        }
        let mut value = parse_header_value(value)?;
        if is_conventionally_secret_header(&name) {
            value.set_sensitive(true);
        }
        headers.insert(name, value);
    }
    Ok(())
}

fn register_auth_redaction(redactor: &mut Redactor, authentication: &Authentication) -> Result<()> {
    match authentication {
        Authentication::None => {}
        Authentication::FmpHeader(secret) | Authentication::Bearer(secret) => {
            redactor.add_secret(secret);
        }
        Authentication::FmpQuery(secret) => {
            redactor.add_secret(secret);
            redactor
                .add_secret_query_name("apikey")
                .map_err(|_| Error::configuration("invalid built-in secret query name"))?;
        }
        Authentication::CustomHeader { name, secret, .. } => {
            redactor.add_secret(secret);
            redactor.add_secret_header_name(name.clone()).map_err(|_| {
                Error::configuration_with_kind(
                    ConfigurationErrorKind::InvalidHeaderName,
                    "custom secret header name is invalid",
                )
            })?;
        }
        Authentication::CustomQuery { name, secret } => {
            redactor.add_secret(secret);
            redactor.add_secret_query_name(name.clone()).map_err(|_| {
                Error::configuration_with_kind(
                    ConfigurationErrorKind::InvalidQueryName,
                    "custom secret query name is invalid",
                )
            })?;
        }
    }
    Ok(())
}

fn is_conventionally_secret_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "authorization" | "proxy-authorization" | "apikey" | "x-api-key"
    )
}

fn reserved_header_names() -> BTreeSet<String> {
    [
        "apikey",
        "authorization",
        "connection",
        "content-length",
        "cookie",
        "host",
        "http2-settings",
        "keep-alive",
        "proxy-authorization",
        "proxy-connection",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
        "user-agent",
        "x-api-key",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn is_unsafe_transport_header(name: &HeaderName) -> bool {
    matches!(
        name.as_str(),
        "connection"
            | "content-length"
            | "host"
            | "http2-settings"
            | "keep-alive"
            | "proxy-authorization"
            | "proxy-connection"
            | "te"
            | "trailer"
            | "transfer-encoding"
            | "upgrade"
            | "user-agent"
    )
}

fn is_redirect(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}

fn same_origin(left: &Url, right: &Url) -> bool {
    left.scheme() == right.scheme()
        && left.host() == right.host()
        && left.port_or_known_default() == right.port_or_known_default()
}

fn status_error<R>(
    endpoint: &'static str,
    response: TransportResponse,
    redactor: &Redactor,
) -> Result<R> {
    Err(Error::status(
        endpoint,
        response.status(),
        Some(safe_body(response.body(), redactor)),
    )
    .with_headers(safe_headers(&response, redactor)))
}

fn safe_headers(response: &TransportResponse, redactor: &Redactor) -> SafeHeaders {
    SafeHeaders::from_header_map(response.headers(), redactor)
}

fn safe_body(body: &[u8], redactor: &Redactor) -> SafeBody {
    SafeBody::new(&String::from_utf8_lossy(body), redactor)
}

/// Builds the body of a decode error, summarizing it for binary endpoints so
/// raw payload bytes never reach a diagnostic (#411).
fn error_body(
    expected: ExpectedContentType,
    body: &[u8],
    content_type: Option<&str>,
    redactor: &Redactor,
) -> SafeBody {
    if !expected.is_binary() {
        return safe_body(body, redactor);
    }
    let summary = match content_type {
        Some(content_type) => format!(
            "<binary body omitted: {} bytes, content-type {content_type}>",
            body.len()
        ),
        None => format!("<binary body omitted: {} bytes>", body.len()),
    };
    SafeBody::new(&summary, redactor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_reserves_tokio_deadlines_for_the_built_in_executor() {
        let built_in = Client::builder()
            .base_url("https://example.test")
            .build()
            .unwrap();
        let injected = Client::builder()
            .base_url("https://example.test")
            .executor(Arc::new(
                ReqwestExecutor::new(DEFAULT_CONNECT_TIMEOUT).unwrap(),
            ))
            .build()
            .unwrap();

        assert_eq!(built_in.inner.deadline_backend, DeadlineBackend::Tokio);
        assert_eq!(injected.inner.deadline_backend, DeadlineBackend::Thread);
    }

    #[tokio::test]
    async fn tokio_deadline_backend_times_out() {
        assert!(
            with_timeout(
                DeadlineBackend::Tokio,
                Duration::from_millis(1),
                std::future::pending::<()>(),
            )
            .await
            .is_none()
        );
    }

    #[test]
    fn request_headers_override_defaults_except_reserved_fields() {
        let protected = reserved_header_names();
        let mut headers =
            parse_headers(&[("x-mode".to_owned(), "default".to_owned())], &protected).unwrap();
        merge_request_headers(&mut headers, &[("X-Mode", "request")], &protected).unwrap();
        assert_eq!(headers["x-mode"], "request");

        for name in [
            "AUTHORIZATION",
            "apikey",
            "X-API-KEY",
            "Cookie",
            "Host",
            "Content-Length",
            "Connection",
            "Transfer-Encoding",
            "User-Agent",
        ] {
            let error = merge_request_headers(&mut headers, &[(name, "replacement")], &protected)
                .unwrap_err();
            assert_eq!(
                error.configuration_kind(),
                Some(ConfigurationErrorKind::ProtectedFieldCollision)
            );
        }
    }

    #[test]
    fn origin_matching_normalizes_default_ports_but_not_scheme() {
        let https_default = Url::parse("https://example.test/path").unwrap();
        let https_explicit = Url::parse("https://example.test:443/other").unwrap();
        let different_port = Url::parse("https://example.test:444/path").unwrap();
        let different_scheme = Url::parse("http://example.test:443/path").unwrap();
        assert!(same_origin(&https_default, &https_explicit));
        assert!(!same_origin(&https_default, &different_port));
        assert!(!same_origin(&https_default, &different_scheme));
    }

    #[test]
    fn leading_slashes_never_reset_the_base_path() {
        let base = Url::parse("https://example.test/gateway").unwrap();
        let url = build_endpoint_url(&base, "/stable/", "/quote-short").unwrap();
        assert_eq!(
            url.as_str(),
            "https://example.test/gateway/stable/quote-short"
        );
    }

    fn env_client(api_key: Option<&str>, base_url: Option<&str>) -> Result<Client> {
        ClientBuilder::from_env_values(api_key.map(str::to_owned), base_url.map(str::to_owned))?
            .build()
    }

    #[test]
    fn env_constructor_targets_a_proxy_base_with_the_apikey_header() {
        let client = env_client(Some("vk_example"), Some("https://valet.bitbrew.app/fmp")).unwrap();
        let url =
            build_endpoint_url(&client.inner.base_url, &client.inner.path_prefix, "quote").unwrap();
        assert_eq!(url.as_str(), "https://valet.bitbrew.app/fmp/stable/quote");
        let (name, value) = client.inner.auth.header.as_ref().unwrap();
        assert_eq!(name.as_str(), "apikey");
        assert_eq!(value, "vk_example");
        assert!(client.inner.auth.query.is_none());
    }

    #[test]
    fn env_constructor_without_a_base_url_keeps_the_fmp_origin() {
        let client = env_client(Some("vk_example"), None).unwrap();
        let url =
            build_endpoint_url(&client.inner.base_url, &client.inner.path_prefix, "quote").unwrap();
        assert_eq!(
            url.as_str(),
            "https://financialmodelingprep.com/stable/quote"
        );
    }

    #[test]
    fn env_constructor_reports_typed_configuration_errors() {
        for (api_key, base_url, kind) in [
            (None, None, ConfigurationErrorKind::MissingCredential),
            (
                None,
                Some("https://valet.bitbrew.app/fmp"),
                ConfigurationErrorKind::MissingCredential,
            ),
            (
                Some("vk_example"),
                Some("valet.bitbrew.app/fmp"),
                ConfigurationErrorKind::InvalidBaseUrl,
            ),
            (
                Some("vk_example"),
                Some("https://user:pass@valet.bitbrew.app/fmp"),
                ConfigurationErrorKind::UnsafeBaseUrl,
            ),
            (
                Some("vk_example"),
                Some("http://example.test/fmp"),
                ConfigurationErrorKind::InsecureAuthentication,
            ),
        ] {
            let error = env_client(api_key, base_url).unwrap_err();
            assert_eq!(error.configuration_kind(), Some(kind));
            assert!(!error.to_string().contains("vk_example"));
        }
    }

    #[test]
    fn env_constructor_keeps_the_loopback_and_dangerous_http_rules() {
        assert!(env_client(Some("vk_example"), Some("http://127.0.0.1:8080/fmp")).is_ok());
        let builder = ClientBuilder::from_env_values(
            Some("vk_example".to_owned()),
            Some("http://example.test/fmp".to_owned()),
        )
        .unwrap();
        assert!(
            builder
                .danger_allow_insecure_authentication(true)
                .build()
                .is_ok()
        );
    }

    #[test]
    fn env_constructor_rejects_a_second_authentication() {
        let error = ClientBuilder::from_env_values(Some("vk_example".to_owned()), None)
            .unwrap()
            .authentication(Authentication::bearer("other"))
            .build()
            .unwrap_err();
        assert_eq!(
            error.configuration_kind(),
            Some(ConfigurationErrorKind::ConflictingAuthentication)
        );
    }

    #[test]
    fn default_response_limit_is_the_documented_finite_value() {
        assert_eq!(DEFAULT_MAX_RESPONSE_BODY_BYTES, 64 * 1024 * 1024);
        assert_eq!(DEFAULT_BULK_MAX_RESPONSE_BODY_BYTES, 256 * 1024 * 1024);
        assert_eq!(ClientBuilder::default().max_response_body_bytes, None);
    }

    #[test]
    fn default_timeouts_are_the_documented_values() {
        assert_eq!(DEFAULT_TIMEOUT, Duration::from_secs(30));
        assert_eq!(DEFAULT_BULK_TIMEOUT, Duration::from_secs(600));
        assert_eq!(ClientBuilder::default().timeout, None);
    }
}
