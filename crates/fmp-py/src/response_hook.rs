//! The `on_response` callback of `FmpClient`.
//!
//! The libfmp observer runs inside the request, on whichever thread polls it,
//! without the GIL. It only records what it saw into a per-call task-local
//! slot that [`capture`] scopes around one call; [`deliver`] then runs the
//! Python callback on the calling thread, with the GIL, once the call is done.
//! Concurrent calls each own their slot, so no thread sees another's headers.

use std::{
    collections::BTreeMap,
    future::Future,
    mem,
    sync::{Arc, Mutex, PoisonError},
};

use libfmp::ResponseInfo;
use pyo3::{prelude::*, types::PyAny};

/// One successful response as the callback receives it.
pub(crate) struct Seen {
    endpoint: &'static str,
    status: u16,
    headers: BTreeMap<String, String>,
}

type Slot = Arc<Mutex<Vec<Seen>>>;

tokio::task_local! {
    static SEEN: Slot;
}

/// The libfmp observer: records one success into the current call's slot.
///
/// Header names are lowercase and the first value of a repeated name wins,
/// the same rule as the error `headers` attribute. Outside [`capture`] (a
/// client without `on_response`) it does nothing.
pub(crate) fn record(info: &ResponseInfo<'_>) {
    let _ = SEEN.try_with(|slot| {
        let mut headers = BTreeMap::new();
        for (name, value) in info.headers.iter() {
            headers
                .entry(name.to_owned())
                .or_insert_with(|| value.to_owned());
        }
        slot.lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(Seen {
                endpoint: info.endpoint,
                status: info.status,
                headers,
            });
    });
}

/// Runs `future` with a fresh slot and returns its output with every success
/// recorded during it.
pub(crate) async fn capture<Fut: Future>(future: Fut) -> (Fut::Output, Vec<Seen>) {
    let slot = Slot::default();
    let output = SEEN.scope(Arc::clone(&slot), future).await;
    let seen = mem::take(&mut *slot.lock().unwrap_or_else(PoisonError::into_inner));
    (output, seen)
}

/// Calls `callback(endpoint_id, status, headers)` for each recorded success,
/// re-acquiring the GIL on the calling thread. The first exception it raises
/// is returned, so the endpoint method raises it instead of returning data.
pub(crate) fn deliver(callback: &Py<PyAny>, seen: Vec<Seen>) -> PyResult<()> {
    if seen.is_empty() {
        return Ok(());
    }
    Python::attach(|py| {
        for success in seen {
            callback.call1(py, (success.endpoint, success.status, success.headers))?;
        }
        Ok(())
    })
}
