#[allow(dead_code)] // Only the fixture executor is used here.
mod support;

use std::{sync::Arc, time::Duration};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::economics::EconomicIndicatorsQuery,
    error::ErrorCategory,
    http::{HeaderMap, HeaderName, HeaderValue, header::CONTENT_TYPE},
    query::EconomicIndicator,
    transport::TransportResponse,
};

use support::FixtureExecutor;

const VALET_HEADERS: &[(&str, &str)] = &[
    ("Retry-After", "12"),
    ("X-Proxy-Key-Id", "vk_123"),
    ("X-Proxy-Error", "rate_limited"),
    ("Set-Cookie", "session=cookie-secret"),
    ("Authorization", "Bearer header-secret"),
    ("apikey", "query-secret"),
    ("Cookie", "cookie=jar-secret"),
    ("X-Unrelated", "unrelated-value"),
    ("X-Proxy-Burst-Remaining", "0"),
    ("X-Proxy-Daily-Remaining", "250"),
    ("X-Proxy-Bytes-30d", "1048576"),
    ("X-RateLimit-Remaining", "0"),
];

fn header_map(pairs: &[(&str, &str)]) -> HeaderMap {
    let mut headers = HeaderMap::new();
    for (name, value) in pairs {
        headers.append(
            HeaderName::from_bytes(name.as_bytes()).unwrap(),
            HeaderValue::from_str(value).unwrap(),
        );
    }
    headers
}

async fn economic_indicators(response: TransportResponse) -> libfmp::Error {
    let executor = Arc::new(FixtureExecutor::new([response]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::None)
        .executor(executor)
        .build()
        .unwrap();
    client
        .economic_indicators(EconomicIndicatorsQuery::new(EconomicIndicator::Gdp))
        .await
        .unwrap_err()
}

fn assert_only_allowlisted(error: &libfmp::Error) {
    let kept: Vec<_> = error.headers().iter().collect();
    assert_eq!(
        kept,
        [
            ("retry-after", "12"),
            ("x-proxy-key-id", "vk_123"),
            ("x-proxy-error", "rate_limited"),
            ("x-proxy-burst-remaining", "0"),
            ("x-proxy-daily-remaining", "250"),
            ("x-proxy-bytes-30d", "1048576"),
            ("x-ratelimit-remaining", "0"),
        ]
    );
    for name in [
        "set-cookie",
        "authorization",
        "apikey",
        "cookie",
        "x-unrelated",
    ] {
        assert_eq!(error.headers().get(name), None, "{name}");
    }
    let debug = format!("{error:?}");
    for secret in [
        "cookie-secret",
        "header-secret",
        "query-secret",
        "jar-secret",
        "unrelated-value",
    ] {
        assert!(!debug.contains(secret), "{secret}");
    }
}

#[tokio::test]
async fn status_error_keeps_retry_after_and_proxy_headers_only() {
    let response = TransportResponse::new(429, header_map(VALET_HEADERS), "slow down");
    let error = economic_indicators(response).await;

    assert_eq!(error.category(), ErrorCategory::Status);
    assert_eq!(error.status_code(), Some(429));
    assert_eq!(error.retry_after(), Some(Duration::from_secs(12)));
    assert_eq!(error.proxy_error(), Some("rate_limited"));
    assert_eq!(error.headers().get("X-PROXY-KEY-ID"), Some("vk_123"));
    assert_only_allowlisted(&error);
    assert_eq!(
        error.to_string(),
        "provider returned HTTP status 429 (endpoint: economic-indicators): slow down"
    );
}

#[tokio::test]
async fn provider_message_keeps_allowlisted_headers() {
    let mut headers = header_map(VALET_HEADERS);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let error = economic_indicators(TransportResponse::new(200, headers, "Invalid name")).await;

    assert_eq!(error.category(), ErrorCategory::Status);
    assert_eq!(error.retry_after(), Some(Duration::from_secs(12)));
    assert_eq!(error.proxy_error(), Some("rate_limited"));
    assert_only_allowlisted(&error);
}

#[tokio::test]
async fn status_error_without_allowlisted_headers_is_empty() {
    let response = TransportResponse::new(500, header_map(&[("Set-Cookie", "a=b")]), "");
    let error = economic_indicators(response).await;

    assert!(error.headers().is_empty());
    assert_eq!(error.retry_after(), None);
    assert_eq!(error.proxy_error(), None);
}
