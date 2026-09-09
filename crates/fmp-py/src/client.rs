//! The hand-written `FmpClient` core: construction and transport
//! configuration. The per-domain getters (`client.quote`) are generated into
//! `client_namespaces.rs`, a second `#[pymethods]` block on the same class.

use std::{sync::Arc, time::Duration};

use libfmp::{
    ClientBuilder,
    config::{Authentication, RedirectPolicy, fmp_api_key_from_env},
    error::ConfigurationErrorKind,
};
use pyo3::{prelude::*, types::PyDict};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};

use crate::errors::to_py_error;

fn invalid_configuration(message: &'static str) -> PyErr {
    to_py_error(libfmp::Error::configuration(message))
}

/// The message raised when neither `token` nor `FMP_API_KEY` supplies a
/// credential for the default FMP host.
const MISSING_TOKEN_MESSAGE: &str = "no token given and FMP_API_KEY is not set";

/// Resolves the credential: an explicit `token` always wins, `auth_mode="none"`
/// never consults the environment, and every other mode falls back to
/// `FMP_API_KEY`.
fn resolve_token(mode: Option<&str>, token: Option<String>) -> Option<String> {
    match (mode, token) {
        (_, Some(token)) => Some(token),
        (Some("none"), None) => None,
        (_, None) => fmp_api_key_from_env(),
    }
}

/// Names `FMP_API_KEY` when the default host was left without a credential
/// and the caller did not select `auth_mode="none"` explicitly.
fn build_error(auth_mode: Option<&str>, error: libfmp::Error) -> PyErr {
    if auth_mode.is_none()
        && error.configuration_kind() == Some(ConfigurationErrorKind::MissingCredential)
    {
        invalid_configuration(MISSING_TOKEN_MESSAGE)
    } else {
        to_py_error(error)
    }
}

fn required_token(token: Option<String>) -> PyResult<String> {
    token.ok_or_else(|| invalid_configuration("the selected authentication mode requires token"))
}

fn reject_auth_options(
    token: &Option<String>,
    name: &Option<String>,
    prefix: &Option<String>,
) -> PyResult<()> {
    if token.is_some() || name.is_some() || prefix.is_some() {
        Err(invalid_configuration(
            "auth_mode='none' cannot be combined with token, auth_name, or auth_prefix",
        ))
    } else {
        Ok(())
    }
}

fn reject_name_and_prefix(name: &Option<String>, prefix: &Option<String>) -> PyResult<()> {
    if name.is_some() || prefix.is_some() {
        Err(invalid_configuration(
            "the selected authentication mode cannot be combined with auth_name or auth_prefix",
        ))
    } else {
        Ok(())
    }
}

fn authentication(
    mode: Option<&str>,
    token: Option<String>,
    name: Option<String>,
    prefix: Option<String>,
) -> PyResult<Authentication> {
    let token = resolve_token(mode, token);
    let mode = mode.unwrap_or(if token.is_some() {
        "fmp_header"
    } else {
        "none"
    });
    match mode {
        "none" => {
            reject_auth_options(&token, &name, &prefix)?;
            Ok(Authentication::None)
        }
        "fmp_header" => {
            reject_name_and_prefix(&name, &prefix)?;
            Ok(Authentication::fmp_header(required_token(token)?))
        }
        "fmp_query" => {
            reject_name_and_prefix(&name, &prefix)?;
            Ok(Authentication::fmp_query(required_token(token)?))
        }
        "bearer" => {
            reject_name_and_prefix(&name, &prefix)?;
            Ok(Authentication::bearer(required_token(token)?))
        }
        "custom_header" => {
            let name = name.ok_or_else(|| {
                invalid_configuration("auth_mode='custom_header' requires auth_name")
            })?;
            Ok(Authentication::custom_header(
                name,
                prefix,
                required_token(token)?,
            ))
        }
        "custom_query" => {
            if prefix.is_some() {
                return Err(invalid_configuration(
                    "auth_mode='custom_query' cannot be combined with auth_prefix",
                ));
            }
            let name = name.ok_or_else(|| {
                invalid_configuration("auth_mode='custom_query' requires auth_name")
            })?;
            Ok(Authentication::custom_query(name, required_token(token)?))
        }
        _ => Err(invalid_configuration(
            "auth_mode must be one of: none, fmp_header, fmp_query, bearer, custom_header, custom_query",
        )),
    }
}

fn positive_duration(value: f64, field: &'static str) -> PyResult<Duration> {
    let message = match field {
        "timeout" => "timeout must be a finite number greater than zero",
        _ => "connect_timeout must be a finite number greater than zero",
    };
    if value <= 0.0 {
        return Err(invalid_configuration(message));
    }
    Duration::try_from_secs_f64(value).map_err(|_| invalid_configuration(message))
}

/// A synchronous FMP client with proxy-ready transport configuration.
///
/// When `token` is omitted, the `FMP_API_KEY` environment variable is read
/// instead (unset, empty, or whitespace-only counts as absent); an explicit
/// `token` always wins, and `auth_mode="none"` ignores the variable. A
/// credential without `auth_mode` selects FMP's exact `apikey` header; the
/// other modes combine with the variable as they do with `token`. Omitting
/// both selects no auth, which is valid only with a custom base URL: against
/// the default host the constructor raises `FmpConfigError` naming
/// `FMP_API_KEY`.
/// `timeout` and `connect_timeout` are positive finite numbers of seconds.
/// `max_response_body_bytes` bounds each buffered response. Authenticated
/// non-loopback HTTP requires `danger_allow_insecure_authentication=True`.
/// Redirects are either disabled or restricted to the same origin.
#[gen_stub_pyclass]
#[pyclass(module = "fmp._native", frozen)]
pub(crate) struct FmpClient {
    pub(crate) builder: Arc<ClientBuilder>,
}

#[gen_stub_pymethods]
#[pymethods]
impl FmpClient {
    #[new]
    #[pyo3(signature = (*, token=None, base_url=None, path_prefix=None, auth_mode=None, auth_name=None, auth_prefix=None, headers=None, timeout=None, connect_timeout=None, max_response_body_bytes=None, danger_allow_insecure_authentication=false, follow_redirects=None))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        token: Option<String>,
        base_url: Option<String>,
        path_prefix: Option<String>,
        auth_mode: Option<&str>,
        auth_name: Option<String>,
        auth_prefix: Option<String>,
        headers: Option<&Bound<'_, PyDict>>,
        timeout: Option<f64>,
        connect_timeout: Option<f64>,
        max_response_body_bytes: Option<usize>,
        danger_allow_insecure_authentication: bool,
        follow_redirects: Option<bool>,
    ) -> PyResult<Self> {
        let auth = authentication(auth_mode, token, auth_name, auth_prefix)?;
        let mut builder = ClientBuilder::default().authentication(auth);

        if let Some(base_url) = base_url {
            builder = builder.base_url(base_url);
        }
        if let Some(path_prefix) = path_prefix {
            builder = builder.path_prefix(path_prefix);
        }
        if let Some(headers) = headers {
            for (name, value) in headers.iter() {
                builder =
                    builder.default_header(name.extract::<String>()?, value.extract::<String>()?);
            }
        }
        if let Some(timeout) = timeout {
            builder = builder.timeout(positive_duration(timeout, "timeout")?);
        }
        if let Some(connect_timeout) = connect_timeout {
            builder =
                builder.connect_timeout(positive_duration(connect_timeout, "connect_timeout")?);
        }
        if let Some(max_response_body_bytes) = max_response_body_bytes {
            builder = builder.max_response_body_bytes(max_response_body_bytes);
        }
        builder =
            builder.danger_allow_insecure_authentication(danger_allow_insecure_authentication);
        if let Some(follow_redirects) = follow_redirects {
            builder = builder.redirect_policy(if follow_redirects {
                RedirectPolicy::SameOrigin
            } else {
                RedirectPolicy::None
            });
        }

        builder
            .clone()
            .build()
            .map_err(|error| build_error(auth_mode, error))?;
        Ok(Self {
            builder: Arc::new(builder),
        })
    }
}
