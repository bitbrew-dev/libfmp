mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        market::{biggest_gainers, biggest_losers, most_actives},
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::market::MarketMover,
    transport::HttpMethod,
};

use support::{FixtureExecutor, json_fixture};

const GAINERS: &[u8] = include_bytes!("fixtures/biggest_gainers.json");
const LOSERS: &[u8] = include_bytes!("fixtures/biggest_losers.json");
const ACTIVES: &[u8] = include_bytes!("fixtures/most_actives.json");

#[test]
fn descriptors_use_exact_queryless_paths_shared_bare_rows_and_only_documented_metadata() {
    let gainers = biggest_gainers();
    let losers = biggest_losers();
    let actives = most_actives();

    assert_facts(&gainers, "biggest-gainers");
    assert_facts(&losers, "biggest-losers");
    assert_facts(&actives, "most-actives");
    assert_response_types(&gainers, &losers, &actives);
}

fn assert_facts(endpoint: &EndpointSpec<(), Vec<MarketMover>>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(endpoint.query(), &());
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::UsOnly
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

fn assert_response_types(
    _: &EndpointSpec<(), Vec<MarketMover>>,
    _: &EndpointSpec<(), Vec<MarketMover>>,
    _: &EndpointSpec<(), Vec<MarketMover>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_exact_queryless_urls_headers_and_source_fixtures() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(GAINERS),
        json_fixture(LOSERS),
        json_fixture(ACTIVES),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "market movers")
        .executor(executor.clone())
        .build()
        .unwrap();

    let gainers = client.biggest_gainers().await.unwrap();
    let losers = client.biggest_losers().await.unwrap();
    let actives = client.most_actives().await.unwrap();

    assert_eq!(gainers[0].symbol.as_str(), "MOTS");
    assert_eq!(gainers[0].price, 0.0002);
    assert_eq!(gainers[0].change, 0.0001);
    assert_eq!(gainers[0].changes_percentage, 100.0);
    assert_eq!(losers[0].symbol.as_str(), "SPEC");
    assert_eq!(losers[0].change, -0.002);
    assert_eq!(losers[0].changes_percentage, -90.90909);
    assert_eq!(actives[0].symbol.as_str(), "LUCY");
    assert_eq!(actives[0].price, 1.85);
    assert_eq!(actives[0].exchange.as_str(), "NASDAQ");

    let requests = executor.requests();
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "market movers"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/biggest-gainers",
            "https://proxy.example/router/gateway/stable/biggest-losers",
            "https://proxy.example/router/gateway/stable/most-actives",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_add_nothing_but_transport_authentication() {
    for (authentication, suffix, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "?apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([
            json_fixture(GAINERS),
            json_fixture(LOSERS),
            json_fixture(ACTIVES),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client.biggest_gainers().await.unwrap();
        client.biggest_losers().await.unwrap();
        client.most_actives().await.unwrap();

        let requests = executor.requests();
        assert_eq!(
            requests
                .iter()
                .map(|request| request.expose_url().as_str())
                .collect::<Vec<_>>(),
            [
                format!("https://financialmodelingprep.com/stable/biggest-gainers{suffix}"),
                format!("https://financialmodelingprep.com/stable/biggest-losers{suffix}"),
                format!("https://financialmodelingprep.com/stable/most-actives{suffix}"),
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
async fn malformed_non_array_responses_keep_each_endpoint_identity() {
    let executor = Arc::new(FixtureExecutor::new(
        std::iter::repeat_with(|| json_fixture(b"{}")).take(3),
    ));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();

    let errors = [
        client.biggest_gainers().await.unwrap_err(),
        client.biggest_losers().await.unwrap_err(),
        client.most_actives().await.unwrap_err(),
    ];

    for (error, endpoint) in
        errors
            .iter()
            .zip(["biggest-gainers", "biggest-losers", "most-actives"])
    {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}
