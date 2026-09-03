use std::time::Duration;

use libfmp::{
    Client, ClientBuilder,
    config::{Authentication, RedirectPolicy},
    types::Ticker,
};
use pyo3::{prelude::*, types::PyDict};
use tokio::runtime::Builder as RuntimeBuilder;

use crate::{errors::to_py_error, quote::QuoteShort};

fn runtime() -> PyResult<tokio::runtime::Runtime> {
    RuntimeBuilder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|_| {
            to_py_error(libfmp::Error::configuration(
                "the fmp runtime could not be constructed",
            ))
        })
}

fn invalid_configuration(message: &'static str) -> PyErr {
    to_py_error(libfmp::Error::configuration(message))
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
/// Supplying `token` without `auth_mode` selects FMP's exact `apikey` header;
/// omitting both selects no auth, which is valid only with a custom base URL.
/// `timeout` and `connect_timeout` are positive finite numbers of seconds.
/// Redirects are either disabled or restricted to the same origin.
#[pyclass(module = "fmp.client", frozen)]
pub(crate) struct FmpClient {
    client: Client,
}

#[pymethods]
impl FmpClient {
    #[new]
    #[pyo3(signature = (*, token=None, base_url=None, path_prefix=None, auth_mode=None, auth_name=None, auth_prefix=None, headers=None, timeout=None, connect_timeout=None, follow_redirects=None))]
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
        if let Some(follow_redirects) = follow_redirects {
            builder = builder.redirect_policy(if follow_redirects {
                RedirectPolicy::SameOrigin
            } else {
                RedirectPolicy::None
            });
        }

        builder
            .build()
            .map(|client| Self { client })
            .map_err(to_py_error)
    }

    /// Retrieve the documented bare quote-short array without changing its shape.
    fn quote_short(&self, py: Python<'_>, symbol: &str) -> PyResult<Vec<QuoteShort>> {
        let symbol = Ticker::new(symbol)
            .map_err(libfmp::Error::from)
            .map_err(to_py_error)?;
        let client = self.client.clone();
        let runtime = runtime()?;
        let result = py.detach(move || runtime.block_on(client.quote_short(symbol)));

        result
            .map(|rows| rows.into_iter().map(QuoteShort::from).collect())
            .map_err(to_py_error)
    }
}
