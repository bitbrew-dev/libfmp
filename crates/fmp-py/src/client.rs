use std::{
    cell::RefCell,
    collections::HashMap,
    future::Future,
    mem, process,
    sync::{Arc, Weak},
    time::Duration,
};

use libfmp::{
    Client, ClientBuilder,
    config::{Authentication, RedirectPolicy},
    types::Ticker,
};
use pyo3::{prelude::*, types::PyDict};
use tokio::runtime::Builder as RuntimeBuilder;

use crate::{errors::to_py_error, quote::QuoteShort};

struct CachedClient {
    owner: Weak<ClientBuilder>,
    client: Client,
}

struct RuntimeState {
    owner_pid: u32,
    clients: HashMap<usize, CachedClient>,
    runtime: Option<tokio::runtime::Runtime>,
}

impl RuntimeState {
    fn new() -> Self {
        Self {
            owner_pid: process::id(),
            clients: HashMap::new(),
            runtime: None,
        }
    }

    fn reset_after_fork(&mut self) {
        let current_pid = process::id();
        if self.owner_pid != current_pid {
            // Fork removes the runtime workers that own both the Tokio driver
            // and Reqwest pool tasks. Their destructors may wait, panic, or
            // abort in the child, so leak only this unreachable inherited state.
            let inherited_clients = mem::take(&mut self.clients);
            mem::forget(inherited_clients);
            if let Some(inherited_runtime) = self.runtime.take() {
                mem::forget(inherited_runtime);
            }
            self.owner_pid = current_pid;
        }
    }

    fn client(&mut self, builder: &Arc<ClientBuilder>) -> PyResult<Client> {
        self.clients
            .retain(|_, cached| cached.owner.upgrade().is_some());
        let id = Arc::as_ptr(builder) as usize;
        if let Some(cached) = self.clients.get(&id) {
            return Ok(cached.client.clone());
        }
        let client = builder.as_ref().clone().build().map_err(to_py_error)?;
        self.clients.insert(
            id,
            CachedClient {
                owner: Arc::downgrade(builder),
                client: client.clone(),
            },
        );
        Ok(client)
    }

    fn runtime(&mut self) -> PyResult<&tokio::runtime::Runtime> {
        if self.runtime.is_none() {
            let runtime = RuntimeBuilder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .map_err(|_| {
                    to_py_error(libfmp::Error::configuration(
                        "the fmp runtime could not be constructed",
                    ))
                })?;
            self.runtime = Some(runtime);
        }

        self.runtime.as_ref().ok_or_else(|| {
            to_py_error(libfmp::Error::configuration(
                "the fmp runtime is unavailable",
            ))
        })
    }
}

impl Drop for RuntimeState {
    fn drop(&mut self) {
        if self.owner_pid != process::id() {
            let inherited_clients = mem::take(&mut self.clients);
            mem::forget(inherited_clients);
            if let Some(inherited_runtime) = self.runtime.take() {
                mem::forget(inherited_runtime);
            }
        } else {
            // Close pooled clients while their runtime is still alive.
            self.clients.clear();
        }
    }
}

thread_local! {
    static RUNTIME: RefCell<RuntimeState> = RefCell::new(RuntimeState::new());
}

fn block_on<F, Fut, T>(builder: Arc<ClientBuilder>, operation: F) -> PyResult<T>
where
    F: FnOnce(Client) -> Fut,
    Fut: Future<Output = T>,
{
    RUNTIME
        .try_with(|state| {
            let mut state = state.try_borrow_mut().map_err(|_| {
                to_py_error(libfmp::Error::configuration(
                    "the fmp runtime is already in use on this thread",
                ))
            })?;
            state.reset_after_fork();
            let client = state.client(&builder)?;
            Ok(state.runtime()?.block_on(operation(client)))
        })
        .map_err(|_| {
            to_py_error(libfmp::Error::configuration(
                "the fmp runtime is unavailable on this thread",
            ))
        })?
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
/// `max_response_body_bytes` bounds each buffered response.
/// Redirects are either disabled or restricted to the same origin.
#[pyclass(module = "fmp.client", frozen)]
pub(crate) struct FmpClient {
    builder: Arc<ClientBuilder>,
}

#[pymethods]
impl FmpClient {
    #[new]
    #[pyo3(signature = (*, token=None, base_url=None, path_prefix=None, auth_mode=None, auth_name=None, auth_prefix=None, headers=None, timeout=None, connect_timeout=None, max_response_body_bytes=None, follow_redirects=None))]
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
        if let Some(follow_redirects) = follow_redirects {
            builder = builder.redirect_policy(if follow_redirects {
                RedirectPolicy::SameOrigin
            } else {
                RedirectPolicy::None
            });
        }

        builder.clone().build().map_err(to_py_error)?;
        Ok(Self {
            builder: Arc::new(builder),
        })
    }

    /// Retrieve the documented bare quote-short array without changing its shape.
    fn quote_short(&self, py: Python<'_>, symbol: &str) -> PyResult<Vec<QuoteShort>> {
        let symbol = Ticker::new(symbol)
            .map_err(libfmp::Error::from)
            .map_err(to_py_error)?;
        let builder = self.builder.clone();
        let result = py.detach(move || {
            block_on(
                builder,
                |client| async move { client.quote_short(symbol).await },
            )
        })?;

        result
            .map(|rows| rows.into_iter().map(QuoteShort::from).collect())
            .map_err(to_py_error)
    }
}
