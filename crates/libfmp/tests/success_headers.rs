#[allow(dead_code)] // Only the fixture executor is used here.
mod support;

use std::sync::Arc;

use libfmp::{
    Client,
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

fn client(response: TransportResponse) -> Client {
    Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::None)
        .executor(Arc::new(FixtureExecutor::new([response])))
        .build()
        .unwrap()
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
