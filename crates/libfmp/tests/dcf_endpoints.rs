mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        dcf::{DcfQuery, discounted_cash_flow, levered_discounted_cash_flow},
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::dcf::DcfValuation,
    transport::HttpMethod,
    types::Ticker,
};

use support::{FixtureExecutor, json_fixture};

const STANDARD: &[u8] = include_bytes!("fixtures/discounted_cash_flow.json");
const LEVERED: &[u8] = include_bytes!("fixtures/levered_discounted_cash_flow.json");

#[test]
fn descriptors_use_exact_get_paths_ids_shared_query_and_worldwide_metadata() {
    let symbol = Ticker::new("BRK.B / Class A").unwrap();
    let standard_query = DcfQuery::new(symbol.clone());
    let levered_query = DcfQuery::new(symbol);

    assert_eq!(standard_query.symbol().as_str(), "BRK.B / Class A");
    assert_eq!(levered_query.symbol().as_str(), "BRK.B / Class A");
    assert_facts(
        &discounted_cash_flow(standard_query),
        "discounted-cash-flow",
    );
    assert_facts(
        &levered_discounted_cash_flow(levered_query),
        "levered-discounted-cash-flow",
    );
}

fn assert_facts(endpoint: &EndpointSpec<DcfQuery, Vec<DcfValuation>>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn proxy_routes_both_methods_with_exact_query_order_custom_auth_header_and_fixtures() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(STANDARD),
        json_fixture(LEVERED),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("/gateway/stable/")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "dcf")
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    let standard = client.discounted_cash_flow(&symbol).await.unwrap();
    let levered = client.levered_discounted_cash_flow(&symbol).await.unwrap();

    assert_eq!(standard[0].dcf, 147.10881272667325);
    assert_eq!(levered[0].dcf, 140.6429495133426);

    let requests = executor.requests();
    assert_eq!(requests.len(), 2);
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "dcf"
    }));
    assert_eq!(
        request_urls(&requests),
        [
            "https://proxy.example/router/gateway/stable/discounted-cash-flow?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/gateway/stable/levered-discounted-cash-flow?symbol=BRK.B+%2F+Class+A",
        ]
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_preserve_exact_paths_and_symbol_only_query() {
    for (authentication, query_auth, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            false,
            Some("header-secret"),
        ),
        (Authentication::fmp_query("query-secret"), true, None),
    ] {
        let executor = Arc::new(FixtureExecutor::new([
            json_fixture(STANDARD),
            json_fixture(LEVERED),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        let symbol = Ticker::new("AAPL").unwrap();

        client.discounted_cash_flow(&symbol).await.unwrap();
        client.levered_discounted_cash_flow(&symbol).await.unwrap();

        let auth = if query_auth {
            "&apikey=query-secret"
        } else {
            ""
        };
        let requests = executor.requests();
        assert_eq!(
            request_urls(&requests),
            [
                format!(
                    "https://financialmodelingprep.com/stable/discounted-cash-flow?symbol=AAPL{auth}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/levered-discounted-cash-flow?symbol=AAPL{auth}"
                ),
            ]
        );
        for request in requests.iter() {
            match expected_header {
                Some(value) => assert_eq!(request.expose_headers()["apikey"], value),
                None => assert!(!request.expose_headers().contains_key("apikey")),
            }
        }
    }
}

#[tokio::test]
async fn malformed_non_array_responses_keep_both_endpoint_identities() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"{}"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();
    let symbol = Ticker::new("AAPL").unwrap();

    let errors = [
        client.discounted_cash_flow(&symbol).await.unwrap_err(),
        client
            .levered_discounted_cash_flow(&symbol)
            .await
            .unwrap_err(),
    ];

    for (error, id) in errors
        .iter()
        .zip(["discounted-cash-flow", "levered-discounted-cash-flow"])
    {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(id));
        assert_eq!(error.status_code(), Some(200));
    }
}

fn request_urls(requests: &[libfmp::transport::PreparedRequest]) -> Vec<String> {
    requests
        .iter()
        .map(|request| request.expose_url().as_str().to_owned())
        .collect()
}
