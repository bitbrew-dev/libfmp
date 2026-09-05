//! Process-wide, fork-safe async runtime backing the synchronous FMP clients.
//!
//! A single multi-thread Tokio runtime and a builder-keyed client pool are
//! shared by every `FmpClient` in the process. Both live behind [`current`],
//! which rebuilds them the first time it runs in a new process generation so
//! that a runtime inherited across `fork()` is never used or dropped in the
//! child. This topology is deliberately process-global rather than
//! thread-local: it is the shape a future asynchronous client needs, where a
//! spawned future must outlive the calling thread and deliver its result to the
//! Python event loop.

use std::{
    collections::HashMap,
    future::Future,
    mem, process,
    sync::{Arc, Mutex, OnceLock, RwLock, Weak},
};

use libfmp::{Client, ClientBuilder};
use tokio::runtime::{Builder as RuntimeBuilder, Handle, Runtime};

use pyo3::PyResult;

use crate::errors::to_py_error;

fn configuration_error(message: &'static str) -> pyo3::PyErr {
    to_py_error(libfmp::Error::configuration(message))
}

fn poisoned() -> pyo3::PyErr {
    configuration_error("the fmp runtime lock was poisoned by a panicking thread")
}

/// A client built for one `ClientBuilder`, retained while that builder lives.
struct CachedClient {
    owner: Weak<ClientBuilder>,
    client: Client,
}

/// The live runtime and client pool for one process generation.
///
/// One `Inner` is created per process (and recreated once per `fork()` in the
/// child). The `owner_pid` records the process it was built in so that
/// [`current`] can detect an inherited generation and replace it.
struct Inner {
    owner_pid: u32,
    runtime: Runtime,
    clients: Mutex<HashMap<usize, CachedClient>>,
}

impl Inner {
    fn build() -> PyResult<Arc<Self>> {
        let runtime = RuntimeBuilder::new_multi_thread()
            .enable_all()
            .build()
            .map_err(|_| configuration_error("the fmp runtime could not be constructed"))?;
        Ok(Arc::new(Self {
            owner_pid: process::id(),
            runtime,
            clients: Mutex::new(HashMap::new()),
        }))
    }

    /// Return a cached client for `builder`, building and caching one on miss.
    ///
    /// The cache mutex is held only for the lookup and insert, never across the
    /// `block_on` that follows, so concurrent callers on other threads are not
    /// serialised by their network work.
    fn client(&self, builder: &Arc<ClientBuilder>) -> PyResult<Client> {
        let mut clients = self.clients.lock().map_err(|_| poisoned())?;
        clients.retain(|_, cached| cached.owner.upgrade().is_some());
        let id = Arc::as_ptr(builder) as usize;
        if let Some(cached) = clients.get(&id) {
            return Ok(cached.client.clone());
        }
        let client = builder.as_ref().clone().build().map_err(to_py_error)?;
        clients.insert(
            id,
            CachedClient {
                owner: Arc::downgrade(builder),
                client: client.clone(),
            },
        );
        Ok(client)
    }

    /// Drive `future` to completion on the shared runtime.
    ///
    /// The caller must not be a runtime worker thread; [`block_on`] enforces
    /// this before delegating here.
    fn block_on<F, T>(&self, future: F) -> T
    where
        F: Future<Output = T>,
    {
        self.runtime.block_on(future)
    }
}

impl Drop for Inner {
    fn drop(&mut self) {
        if self.owner_pid == process::id() {
            // Close pooled clients while their runtime workers are still alive,
            // matching the original thread-local design so connection pools shut
            // down cleanly in the owning process.
            if let Ok(mut clients) = self.clients.lock() {
                clients.clear();
            }
        }
    }
}

/// The process-global slot. `None` until the first [`current`] call builds it.
static RUNTIME: OnceLock<RwLock<Option<Arc<Inner>>>> = OnceLock::new();

fn slot() -> &'static RwLock<Option<Arc<Inner>>> {
    RUNTIME.get_or_init(|| RwLock::new(None))
}

/// Return this process generation's runtime, rebuilding it after a `fork()`.
///
/// The fast path takes a read lock and clones the live `Arc<Inner>` when its
/// `owner_pid` matches. On a miss (first use, or a runtime inherited by a forked
/// child) it upgrades to a write lock, forgets any inherited generation, and
/// builds a fresh one.
fn current() -> PyResult<Arc<Inner>> {
    let pid = process::id();

    {
        let guard = slot().read().map_err(|_| poisoned())?;
        if let Some(inner) = guard.as_ref()
            && inner.owner_pid == pid
        {
            return Ok(inner.clone());
        }
    }

    let mut guard = slot().write().map_err(|_| poisoned())?;
    if let Some(inner) = guard.as_ref()
        && inner.owner_pid == pid
    {
        return Ok(inner.clone());
    }
    if let Some(inherited) = guard.take() {
        // SAFETY-CRITICAL (not `unsafe`, but load-bearing): this `Arc<Inner>`
        // was inherited across `fork()`. The child did not inherit the runtime's
        // worker threads, so `Runtime::drop` here would block forever (or abort)
        // waiting on threads that do not exist. Leaking the `Arc` guarantees
        // `Inner::drop`, and the `Runtime::drop` inside it, never run in the
        // child. The leak is bounded to one runtime per `fork()`.
        mem::forget(inherited);
    }

    let fresh = Inner::build()?;
    *guard = Some(fresh.clone());
    Ok(fresh)
}

/// Run `operation` against a cached client on the shared runtime.
///
/// Returns a configuration error when called from inside a runtime worker
/// thread, where `Runtime::block_on` would panic. Callers release the GIL (via
/// `Python::detach`) around this so other Python threads make progress.
pub(crate) fn block_on<F, Fut, T>(builder: Arc<ClientBuilder>, operation: F) -> PyResult<T>
where
    F: FnOnce(Client) -> Fut,
    Fut: Future<Output = T>,
{
    if Handle::try_current().is_ok() {
        return Err(configuration_error(
            "cannot call a synchronous fmp method from inside the fmp runtime",
        ));
    }
    let inner = current()?;
    let client = inner.client(&builder)?;
    Ok(inner.block_on(operation(client)))
}

#[cfg(all(test, unix))]
mod fork_tests {
    use super::*;
    use std::time::{Duration, Instant};

    /// A forked child must rebuild the runtime rather than hang on the inherited
    /// one, and the rebuilt runtime must actually execute work.
    #[test]
    fn current_rebuilds_after_fork() {
        let parent = current().expect("parent runtime builds");
        let parent_pid = parent.owner_pid;
        assert_eq!(parent.block_on(async { 1 + 1 }), 2);

        // SAFETY: `fork` in a multi-threaded test binary is only safe if the
        // child touches non-async-signal-safe code at its own risk. The parent
        // runtime's workers are idle (parked) at this point, so no lock this
        // test relies on is held across the fork. The child calls `current`
        // (which forgets the inherited runtime and builds a fresh one) and then
        // `_exit`, never returning to the test harness.
        let pid = unsafe { libc::fork() };
        assert!(pid >= 0, "fork failed");

        if pid == 0 {
            let code = match current() {
                Ok(child) => {
                    let rebuilt = child.owner_pid == process::id() && child.owner_pid != parent_pid;
                    let ran = child.block_on(async { 21 * 2 }) == 42;
                    if rebuilt && ran { 0 } else { 3 }
                }
                Err(_) => 2,
            };
            // SAFETY: `_exit` is async-signal-safe and skips atexit handlers and
            // destructors that would be unsafe to run in the forked child.
            unsafe { libc::_exit(code) };
        }

        let mut status: libc::c_int = 0;
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let waited = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
            if waited == pid {
                break;
            }
            if Instant::now() >= deadline {
                unsafe {
                    libc::kill(pid, libc::SIGKILL);
                    libc::waitpid(pid, &mut status, 0);
                }
                panic!("forked child hung: runtime was not rebuilt after fork");
            }
            std::thread::sleep(Duration::from_millis(20));
        }

        assert!(
            libc::WIFEXITED(status),
            "child terminated abnormally (status {status})"
        );
        assert_eq!(
            libc::WEXITSTATUS(status),
            0,
            "child reported a fork-safety failure"
        );
    }
}
