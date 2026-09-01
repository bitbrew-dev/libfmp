mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        indexes::{
            dow_jones_constituents, historical_dow_jones_constituents,
            historical_nasdaq_constituents, historical_sp500_constituents, nasdaq_constituents,
            sp500_constituents,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::indexes::{HistoricalIndexConstituent, IndexConstituent},
    transport::HttpMethod,
};

use support::{FixtureExecutor, json_fixture};

const SP500: &[u8] = include_bytes!("fixtures/indexes_sp500_constituents.json");
const NASDAQ: &[u8] = include_bytes!("fixtures/indexes_nasdaq_constituents.json");
const DOW_JONES: &[u8] = include_bytes!("fixtures/indexes_dow_jones_constituents.json");
const HISTORICAL_SP500: &[u8] =
    include_bytes!("fixtures/indexes_historical_sp500_constituents.json");
const HISTORICAL_NASDAQ: &[u8] =
    include_bytes!("fixtures/indexes_historical_nasdaq_constituents.json");
const HISTORICAL_DOW_JONES: &[u8] =
    include_bytes!("fixtures/indexes_historical_dow_jones_constituents.json");

fn fixtures() -> [libfmp::transport::TransportResponse; 6] {
    [
        json_fixture(SP500),
        json_fixture(NASDAQ),
        json_fixture(DOW_JONES),
        json_fixture(HISTORICAL_SP500),
        json_fixture(HISTORICAL_NASDAQ),
        json_fixture(HISTORICAL_DOW_JONES),
    ]
}

#[test]
fn descriptors_use_exact_paths_unit_queries_and_no_inferred_metadata() {
    assert_current(&sp500_constituents(), "sp500-constituent");
    assert_current(&nasdaq_constituents(), "nasdaq-constituent");
    assert_current(&dow_jones_constituents(), "dowjones-constituent");
    assert_historical(
        &historical_sp500_constituents(),
        "historical-sp500-constituent",
    );
    assert_historical(
        &historical_nasdaq_constituents(),
        "historical-nasdaq-constituent",
    );
    assert_historical(
        &historical_dow_jones_constituents(),
        "historical-dowjones-constituent",
    );
}

fn assert_current(endpoint: &EndpointSpec<(), Vec<IndexConstituent>>, path: &'static str) {
    assert_common(endpoint, path);
}

fn assert_historical(
    endpoint: &EndpointSpec<(), Vec<HistoricalIndexConstituent>>,
    path: &'static str,
) {
    assert_common(endpoint, path);
}

fn assert_common<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Unspecified
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn proxy_client_routes_all_six_methods_with_custom_auth_headers_and_decodes_rows() {
    let executor = Arc::new(FixtureExecutor::new(fixtures()));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("/gateway/stable/")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "index-constituents")
        .executor(executor.clone())
        .build()
        .unwrap();

    let sp500 = client.sp500_constituents().await.unwrap();
    let nasdaq = client.nasdaq_constituents().await.unwrap();
    let dow_jones = client.dow_jones_constituents().await.unwrap();
    let historical_sp500 = client.historical_sp500_constituents().await.unwrap();
    let historical_nasdaq = client.historical_nasdaq_constituents().await.unwrap();
    let historical_dow_jones = client.historical_dow_jones_constituents().await.unwrap();

    assert_eq!(sp500[0].symbol.as_str(), "HONA");
    assert_eq!(nasdaq[0].date_first_added, None);
    assert_eq!(dow_jones[0].cik.as_str(), "0001652044");
    assert_eq!(
        historical_sp500[0]
            .removed_ticker
            .as_ref()
            .unwrap()
            .as_str(),
        "CAG"
    );
    assert_eq!(historical_nasdaq[0].removed_ticker, None);
    assert_eq!(historical_dow_jones[0].symbol.as_str(), "GOOGL");

    let requests = executor.requests();
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "index-constituents"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/sp500-constituent",
            "https://proxy.example/router/gateway/stable/nasdaq-constituent",
            "https://proxy.example/router/gateway/stable/dowjones-constituent",
            "https://proxy.example/router/gateway/stable/historical-sp500-constituent",
            "https://proxy.example/router/gateway/stable/historical-nasdaq-constituent",
            "https://proxy.example/router/gateway/stable/historical-dowjones-constituent",
        ]
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_apply_to_constituent_routes() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/sp500-constituent",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/sp500-constituent?apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(SP500)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        let rows = client.sp500_constituents().await.unwrap();
        assert_eq!(rows[0].symbol.as_str(), "HONA");

        let requests = executor.requests();
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some(value) => assert_eq!(requests[0].expose_headers()["apikey"], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}

#[tokio::test]
async fn malformed_non_array_responses_keep_all_six_endpoint_identities() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"{}"),
        json_fixture(b"{}"),
        json_fixture(b"{}"),
        json_fixture(b"{}"),
        json_fixture(b"{}"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();

    let errors = [
        client.sp500_constituents().await.unwrap_err(),
        client.nasdaq_constituents().await.unwrap_err(),
        client.dow_jones_constituents().await.unwrap_err(),
        client.historical_sp500_constituents().await.unwrap_err(),
        client.historical_nasdaq_constituents().await.unwrap_err(),
        client
            .historical_dow_jones_constituents()
            .await
            .unwrap_err(),
    ];

    for (error, endpoint) in errors.iter().zip([
        "sp500-constituent",
        "nasdaq-constituent",
        "dowjones-constituent",
        "historical-sp500-constituent",
        "historical-nasdaq-constituent",
        "historical-dowjones-constituent",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}
