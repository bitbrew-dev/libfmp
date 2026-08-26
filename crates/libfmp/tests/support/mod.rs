use std::{
    collections::VecDeque,
    fmt,
    sync::{Mutex, MutexGuard},
};

use http::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use libfmp::transport::{
    ExecutorError, ExecutorFuture, HttpExecutor, PreparedRequest, TransportResponse,
};

/// Deterministic executor shared by endpoint contract and vertical-slice tests.
pub struct FixtureExecutor {
    responses: Mutex<VecDeque<TransportResponse>>,
    requests: Mutex<Vec<PreparedRequest>>,
}

impl FixtureExecutor {
    pub fn new(responses: impl IntoIterator<Item = TransportResponse>) -> Self {
        Self {
            responses: Mutex::new(responses.into_iter().collect()),
            requests: Mutex::new(Vec::new()),
        }
    }

    pub fn requests(&self) -> MutexGuard<'_, Vec<PreparedRequest>> {
        self.requests.lock().expect("request fixture lock poisoned")
    }
}

impl fmt::Debug for FixtureExecutor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("FixtureExecutor(..)")
    }
}

impl HttpExecutor for FixtureExecutor {
    fn execute(&self, request: PreparedRequest) -> ExecutorFuture<'_> {
        Box::pin(async move {
            self.requests
                .lock()
                .expect("request fixture lock poisoned")
                .push(request);
            self.responses
                .lock()
                .expect("response fixture lock poisoned")
                .pop_front()
                .ok_or_else(ExecutorError::new)
        })
    }
}

pub fn fixture_response(content_type: Option<&str>, body: &'static [u8]) -> TransportResponse {
    let mut headers = HeaderMap::new();
    if let Some(content_type) = content_type {
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(content_type).expect("valid fixture content type"),
        );
    }
    TransportResponse::new(200, headers, body)
}

pub fn json_fixture(body: &'static [u8]) -> TransportResponse {
    fixture_response(Some("application/json"), body)
}
