mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        directory::{
            AvailableExchangesQuery, available_countries, available_exchanges,
            available_industries, available_sectors,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    transport::HttpMethod,
};

use support::{FixtureExecutor, json_fixture};

const EXCHANGES: &[u8] = include_bytes!("fixtures/directory_available_exchanges.json");
const SECTORS: &[u8] = include_bytes!("fixtures/directory_available_sectors.json");
const INDUSTRIES: &[u8] = include_bytes!("fixtures/directory_available_industries.json");
const COUNTRIES: &[u8] = include_bytes!("fixtures/directory_available_countries.json");
const EMPTY: &[u8] = include_bytes!("fixtures/directory_empty.json");

#[test]
fn descriptors_use_exact_paths_and_only_documented_metadata() {
    let omitted = AvailableExchangesQuery::new();
    let disabled = AvailableExchangesQuery::new().with_extended(false);
    let enabled = AvailableExchangesQuery::new().with_extended(true);

    assert_eq!(omitted.extended(), None);
    assert_eq!(disabled.extended(), Some(false));
    assert_eq!(enabled.extended(), Some(true));

    let exchange = available_exchanges(omitted);
    assert_endpoint(&exchange, GeographicAvailability::Worldwide);
    assert_endpoint(&available_sectors(), GeographicAvailability::Unspecified);
    assert_endpoint(&available_industries(), GeographicAvailability::Unspecified);
    assert_endpoint(&available_countries(), GeographicAvailability::Unspecified);

    assert_eq!(exchange.id(), "available-exchanges");
    assert_eq!(exchange.relative_path(), "available-exchanges");
    assert_eq!(available_sectors().id(), "available-sectors");
    assert_eq!(available_industries().id(), "available-industries");
    assert_eq!(available_countries().id(), "available-countries");
}

fn assert_endpoint<Q, R>(endpoint: &EndpointSpec<Q, R>, geography: GeographicAvailability) {
    let metadata = endpoint.metadata();
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), endpoint.relative_path());
    assert_eq!(metadata.geography(), geography);
    assert_eq!(metadata.access(), AccessRequirement::Unspecified);
    assert_eq!(metadata.conditional_plan(), None);
    assert_eq!(metadata.realtime(), None);
    assert_eq!(metadata.bounds(), EndpointBounds::new());
}

#[tokio::test]
async fn proxy_client_preserves_omitted_false_and_true_extended_queries() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(EXCHANGES),
        json_fixture(EXCHANGES),
        json_fixture(EXCHANGES),
        json_fixture(SECTORS),
        json_fixture(INDUSTRIES),
        json_fixture(COUNTRIES),
    ]));
    let client = proxy_client(executor.clone());

    assert_eq!(
        client
            .available_exchanges(AvailableExchangesQuery::new())
            .await
            .unwrap()
            .len(),
        1
    );
    client
        .available_exchanges(AvailableExchangesQuery::new().with_extended(false))
        .await
        .unwrap();
    client
        .available_exchanges(AvailableExchangesQuery::new().with_extended(true))
        .await
        .unwrap();
    assert_eq!(client.available_sectors().await.unwrap().len(), 1);
    assert_eq!(client.available_industries().await.unwrap().len(), 1);
    assert_eq!(client.available_countries().await.unwrap().len(), 1);

    let requests = executor.requests();
    let urls = requests
        .iter()
        .map(|request| request.expose_url().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        urls,
        [
            "https://proxy.example/router/stable/available-exchanges",
            "https://proxy.example/router/stable/available-exchanges?extended=false",
            "https://proxy.example/router/stable/available-exchanges?extended=true",
            "https://proxy.example/router/stable/available-sectors",
            "https://proxy.example/router/stable/available-industries",
            "https://proxy.example/router/stable/available-countries",
        ]
    );
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
}

#[tokio::test]
async fn client_preserves_empty_arrays_for_all_four_taxonomy_endpoints() {
    let executor = Arc::new(FixtureExecutor::new(
        std::iter::repeat_with(|| json_fixture(EMPTY)).take(4),
    ));
    let client = proxy_client(executor);

    assert!(
        client
            .available_exchanges(AvailableExchangesQuery::new())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(client.available_sectors().await.unwrap().is_empty());
    assert!(client.available_industries().await.unwrap().is_empty());
    assert!(client.available_countries().await.unwrap().is_empty());
}

#[tokio::test]
async fn direct_fmp_auth_uses_the_same_available_exchanges_contract() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/available-exchanges?extended=true",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/available-exchanges?extended=true&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(EXCHANGES)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .available_exchanges(AvailableExchangesQuery::new().with_extended(true))
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some((name, value)) => assert_eq!(requests[0].expose_headers()[name], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}

fn proxy_client(executor: Arc<FixtureExecutor>) -> Client {
    Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::None)
        .executor(executor)
        .build()
        .unwrap()
}
