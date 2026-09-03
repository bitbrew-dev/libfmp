#[cfg(target_os = "linux")]
use std::{
    future::Future,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread,
    time::Duration,
};

#[cfg(target_os = "linux")]
use libfmp::{
    Client,
    client::EndpointSpec,
    transport::{ExecutorFuture, HttpExecutor, HttpMethod, PreparedRequest},
};

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct PendingExecutor;

#[cfg(target_os = "linux")]
impl HttpExecutor for PendingExecutor {
    fn execute(&self, _request: PreparedRequest) -> ExecutorFuture<'_> {
        Box::pin(std::future::pending())
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug)]
struct ThreadWaker(thread::Thread);

#[cfg(target_os = "linux")]
impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}

#[cfg(target_os = "linux")]
fn block_on<F: Future>(future: F) -> F::Output {
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => thread::park_timeout(Duration::from_millis(1)),
        }
    }
}

#[cfg(target_os = "linux")]
fn pending_request(timeout: Duration) -> bool {
    let endpoint: EndpointSpec<(), serde_json::Value> =
        EndpointSpec::new(HttpMethod::Get, "fork-timeout", "/pending", ());
    let result = block_on(
        Client::builder()
            .base_url("https://example.test")
            .timeout(timeout)
            .executor(Arc::new(PendingExecutor))
            .build()
            .expect("valid fork regression client")
            .execute(&endpoint),
    );
    result
        .as_ref()
        .is_err_and(|error| error.message() == "request deadline exceeded")
}

#[cfg(target_os = "linux")]
fn main() {
    assert!(pending_request(Duration::from_millis(5)));

    // This Linux-only harness-free executable remains single-threaded before
    // `fork`. The first request initializes the parent's deadline mechanism;
    // the child must not inherit a stale process-global timer handle.
    // SAFETY: child execution is confined to this regression and `_exit`.
    let child = unsafe { libc::fork() };
    assert!(child >= 0, "fork failed");
    if child == 0 {
        // SAFETY: the alarm prevents a broken child timer from hanging CI, and
        // `_exit` avoids running inherited process state destructors.
        unsafe {
            libc::alarm(2);
        }
        let passed = pending_request(Duration::from_millis(20));
        unsafe {
            libc::_exit(i32::from(!passed));
        }
    }

    let mut status = 0;
    // SAFETY: `child` is the positive PID returned by `fork`, and `status`
    // remains valid throughout the call.
    let waited = unsafe { libc::waitpid(child, &mut status, 0) };
    assert_eq!(waited, child);
    assert!(
        libc::WIFEXITED(status),
        "child status {status}, signal {}",
        libc::WTERMSIG(status)
    );
    assert_eq!(libc::WEXITSTATUS(status), 0);
}

#[cfg(not(target_os = "linux"))]
fn main() {}
