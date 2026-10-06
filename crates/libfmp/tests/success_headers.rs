#[allow(dead_code)] // Only the fixture executor is used here.
mod support;

use std::sync::{Arc, Mutex};

use libfmp::{
    Client, ClientBuilder,
    config::Authentication,
    endpoints::quote::{QuoteShortQuery, quote_short},
    error::ErrorCategory,
    http::{HeaderMap, HeaderName, HeaderValue},
    transport::TransportResponse,
    types::Ticker,
};

use support::FixtureExecutor;

const BODY: &str = r#"[{"symbol":"AAPL","price":232.8,"change":2.1,"volume":44000000}]"#;

const VALET_HEADERS: &[(&str, &str)] = &[
    ("Content-Type", "application/json"),
    ("X-Proxy-Cache", "HIT"),
    ("Set-Cookie", "session=cookie-secret"),
    ("Authorization", "Bearer header-secret"),
    ("X-Unrelated", "unrelated-value"),
    ("X-Proxy-Daily-Remaining", "42"),
];

fn response(status: u16, pairs: &[(&str, &str)], body: &'static str) -> TransportResponse {
    let mut headers = HeaderMap::new();
    for (name, value) in pairs {
        headers.append(
            HeaderName::from_bytes(name.as_bytes()).unwrap(),
            HeaderValue::from_str(value).unwrap(),
        );
    }
    TransportResponse::new(status, headers, body)
}

fn builder(response: TransportResponse) -> ClientBuilder {
    Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::None)
        .executor(Arc::new(FixtureExecutor::new([response])))
}

fn client(response: TransportResponse) -> Client {
    builder(response).build().unwrap()
}

type Seen = Arc<Mutex<Vec<(&'static str, u16, Vec<(String, String)>)>>>;

/// Builds a client whose observer records every call it sees.
fn observed(response: TransportResponse) -> (Client, Seen) {
    let seen = Seen::default();
    let sink = Arc::clone(&seen);
    let client = builder(response)
        .on_response(move |info| {
            let headers = info
                .headers
                .iter()
                .map(|(name, value)| (name.to_owned(), value.to_owned()))
                .collect();
            sink.lock()
                .unwrap()
                .push((info.endpoint, info.status, headers));
        })
        .build()
        .unwrap();
    (client, seen)
}

#[tokio::test]
async fn execute_with_metadata_keeps_status_and_proxy_headers_only() {
    let client = client(response(200, VALET_HEADERS, BODY));
    let query = QuoteShortQuery::new(Ticker::new("AAPL").unwrap());

    let response = client
        .execute_with_metadata(&quote_short(query))
        .await
        .unwrap();

    assert_eq!(response.status, 200);
    assert_eq!(response.data.len(), 1);
    assert_eq!(response.data[0].symbol.as_str(), "AAPL");
    let kept: Vec<_> = response.headers.iter().collect();
    assert_eq!(
        kept,
        [("x-proxy-cache", "HIT"), ("x-proxy-daily-remaining", "42")]
    );
    assert_eq!(response.headers.get("X-Proxy-Cache"), Some("HIT"));
    let debug = format!("{response:?}");
    for secret in ["cookie-secret", "header-secret", "unrelated-value"] {
        assert!(!debug.contains(secret), "{secret}");
    }
}

#[tokio::test]
async fn execute_returns_the_same_data_without_metadata() {
    let query = QuoteShortQuery::new(Ticker::new("AAPL").unwrap());
    let with_metadata = client(response(200, VALET_HEADERS, BODY))
        .execute_with_metadata(&quote_short(query.clone()))
        .await
        .unwrap();
    let rows = client(response(200, VALET_HEADERS, BODY))
        .execute(&quote_short(query))
        .await
        .unwrap();

    assert_eq!(rows, with_metadata.data);
}

#[tokio::test]
async fn execute_with_metadata_errors_are_unchanged() {
    let failure = response(429, &[("X-Proxy-Daily-Remaining", "0")], "slow down");
    let query = QuoteShortQuery::new(Ticker::new("AAPL").unwrap());

    let error = client(failure)
        .execute_with_metadata(&quote_short(query))
        .await
        .unwrap_err();

    assert_eq!(error.category(), ErrorCategory::Status);
    assert_eq!(error.headers().get("x-proxy-daily-remaining"), Some("0"));
}

#[tokio::test]
async fn observer_sees_endpoint_method_successes() {
    let (client, seen) = observed(response(200, VALET_HEADERS, BODY));

    let rows = client
        .quote_short(Ticker::new("AAPL").unwrap())
        .await
        .unwrap();

    assert_eq!(rows.len(), 1);
    let seen = seen.lock().unwrap();
    let pair = |name: &str, value: &str| (name.to_owned(), value.to_owned());
    assert_eq!(
        *seen,
        [(
            "quote-short",
            200,
            vec![
                pair("x-proxy-cache", "HIT"),
                pair("x-proxy-daily-remaining", "42")
            ]
        )]
    );
}

#[tokio::test]
async fn observer_is_silent_on_errors() {
    for failure in [
        response(429, VALET_HEADERS, "slow down"),
        response(
            200,
            VALET_HEADERS,
            r#"{"Error Message":"Invalid API KEY."}"#,
        ),
        response(200, VALET_HEADERS, r#"[{"symbol":7}]"#),
    ] {
        let (client, seen) = observed(failure);
        let query = QuoteShortQuery::new(Ticker::new("AAPL").unwrap());

        client
            .execute_with_metadata(&quote_short(query))
            .await
            .unwrap_err();

        assert!(seen.lock().unwrap().is_empty());
    }
}
