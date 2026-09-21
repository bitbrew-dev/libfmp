//! Opt-in live tests against the real provider and a production proxy route.
//!
//! Every test here is `#[ignore]` and additionally guarded at runtime, so the
//! default `cargo test -p libfmp` never opens a socket. Run them explicitly:
//!
//! ```console
//! FMP_LIVE_TESTS=1 FMP_API_KEY=... cargo test -p libfmp --test live_opt_in -- --ignored
//! ```
//!
//! The proxy test also needs `FMP_PROXY_BASE_URL` and `FMP_PROXY_TOKEN` and
//! mirrors the README router example: the token is sent as
//! `X-Proxy-Token: Bearer <token>`, `FMP_PROXY_PATH_PREFIX` overrides the
//! `router/stable` prefix, and `FMP_TENANT` adds the `X-Tenant` header when
//! set. Secrets are read from the environment, handed to the client, and never
//! formatted: on failure the assertions prove the error text does not contain
//! them before the test panics with that redacted text.

use std::env;

use libfmp::{Client, config::Authentication, responses::quote::QuoteShort, types::Ticker};

const LIVE_SWITCH: &str = "FMP_LIVE_TESTS";
const API_KEY: &str = "FMP_API_KEY";
const PROXY_BASE_URL: &str = "FMP_PROXY_BASE_URL";
const PROXY_TOKEN: &str = "FMP_PROXY_TOKEN";
const PROXY_PATH_PREFIX: &str = "FMP_PROXY_PATH_PREFIX";
const TENANT: &str = "FMP_TENANT";
const DEFAULT_PROXY_PATH_PREFIX: &str = "router/stable";

fn non_empty(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

/// Returns the values of `names` only when the live switch is on and every
/// variable is set; otherwise explains the skip without printing any value.
fn live_env(names: &[&str]) -> Option<Vec<String>> {
    if non_empty(LIVE_SWITCH).as_deref() != Some("1") {
        eprintln!("skipping live test: {LIVE_SWITCH} is not set to 1");
        return None;
    }
    let mut values = Vec::with_capacity(names.len());
    for name in names {
        match non_empty(name) {
            Some(value) => values.push(value),
            None => {
                eprintln!("skipping live test: {name} is not set");
                return None;
            }
        }
    }
    Some(values)
}

async fn quote_short_without_leaking(client: &Client, secret: &str) -> Vec<QuoteShort> {
    match client.quote_short(Ticker::new("AAPL").unwrap()).await {
        Ok(rows) => rows,
        Err(error) => {
            let diagnostic = format!("{error:?} {error}");
            assert!(
                !diagnostic.contains(secret),
                "error text leaked the configured secret"
            );
            panic!("live quote-short request failed: {diagnostic}");
        }
    }
}

fn assert_apple_rows(rows: &[QuoteShort]) {
    assert!(!rows.is_empty(), "quote-short returned an empty array");
    assert_eq!(rows[0].symbol.as_str(), "AAPL");
}

#[tokio::test]
#[ignore = "live provider call: set FMP_LIVE_TESTS=1 and FMP_API_KEY, then run with --ignored"]
async fn direct_header_authentication_returns_quote_short_rows() {
    let Some(values) = live_env(&[API_KEY]) else {
        return;
    };
    let api_key = &values[0];
    let client = Client::builder()
        .authentication(Authentication::fmp_header(api_key.clone()))
        .build()
        .unwrap();
    assert!(!format!("{client:?}").contains(api_key.as_str()));

    let rows = quote_short_without_leaking(&client, api_key).await;

    assert_apple_rows(&rows);
}

#[tokio::test]
#[ignore = "live proxy call: set FMP_LIVE_TESTS=1, FMP_PROXY_BASE_URL and FMP_PROXY_TOKEN, then run with --ignored"]
async fn production_proxy_route_returns_quote_short_rows() {
    let Some(values) = live_env(&[PROXY_BASE_URL, PROXY_TOKEN]) else {
        return;
    };
    let (base_url, token) = (&values[0], &values[1]);
    let mut builder = Client::builder()
        .base_url(base_url.clone())
        .path_prefix(
            non_empty(PROXY_PATH_PREFIX).unwrap_or_else(|| DEFAULT_PROXY_PATH_PREFIX.to_owned()),
        )
        .authentication(Authentication::custom_header(
            "X-Proxy-Token",
            Some("Bearer ".to_owned()),
            token.clone(),
        ));
    if let Some(tenant) = non_empty(TENANT) {
        builder = builder.default_header("X-Tenant", tenant);
    }
    let client = builder.build().unwrap();
    assert!(!format!("{client:?}").contains(token.as_str()));

    let rows = quote_short_without_leaking(&client, token).await;

    assert_apple_rows(&rows);
}
