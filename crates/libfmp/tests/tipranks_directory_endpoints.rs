mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        tipranks::{TipRanksAnalystsQuery, tipranks_analysts},
    },
    error::ErrorCategory,
    responses::tipranks::TipRanksAnalystProfile,
    transport::HttpMethod,
    types::{Limit, Page, SearchTerm},
};

use support::{FixtureExecutor, json_fixture};

const ANALYSTS: &[u8] = include_bytes!("fixtures/tipranks_analysts.json");

#[test]
fn descriptor_has_exact_path_response_type_and_tipranks_access_metadata() {
    let endpoint = tipranks_analysts(TipRanksAnalystsQuery::new());
    assert_response_type(&endpoint);
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), "tipranks-analysts");
    assert_eq!(endpoint.relative_path(), "tipranks-analysts");
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Unspecified
    );
    assert_eq!(
        endpoint.metadata().access(),
        AccessRequirement::NamedAddOn("TipRanks")
    );
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().realtime(), None);
}

fn assert_response_type(_: &EndpointSpec<TipRanksAnalystsQuery, Vec<TipRanksAnalystProfile>>) {}

#[tokio::test]
async fn custom_proxy_sends_one_exact_full_query_request_with_auth_header_and_fixture() {
    let executor = Arc::new(FixtureExecutor::new([json_fixture(ANALYSTS)]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Bearer ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "tipranks directory")
        .executor(executor.clone())
        .build()
        .unwrap();
    let query = TipRanksAnalystsQuery::new()
        .with_page(Page(0))
        .with_limit(Limit(1_000))
        .with_firm_name(SearchTerm::new("Morgan Stanley / Asia").unwrap())
        .with_analyst_name(SearchTerm::new("Andrew Marok / Exact").unwrap());

    let rows = client.tipranks_analysts(query).await.unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].analyst_name, "Sujeeva De Silva");

    let requests = executor.requests();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].method(), HttpMethod::Get);
    assert_eq!(
        requests[0].expose_headers()["x-router-token"],
        "Bearer proxy-secret"
    );
    assert_eq!(
        requests[0].expose_headers()["x-data-scope"],
        "tipranks directory"
    );
    assert_eq!(
        requests[0].expose_url().as_str(),
        "https://proxy.example/router/gateway/stable/tipranks-analysts?page=0&limit=1000&firmName=Morgan+Stanley+%2F+Asia&analystName=Andrew+Marok+%2F+Exact"
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_empty_query_and_auth_position() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/tipranks-analysts",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/tipranks-analysts?apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(ANALYSTS)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        let rows = client
            .tipranks_analysts(TipRanksAnalystsQuery::new())
            .await
            .unwrap();
        assert_eq!(rows[0].firm_name, "Roth MKM");

        let requests = executor.requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].method(), HttpMethod::Get);
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some(value) => assert_eq!(requests[0].expose_headers()["apikey"], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}

#[tokio::test]
async fn empty_array_succeeds_and_invalid_payloads_keep_endpoint_identity() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"[]"),
        json_fixture(b"{}"),
        json_fixture(b"["),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor.clone())
        .build()
        .unwrap();

    assert!(
        client
            .tipranks_analysts(TipRanksAnalystsQuery::new())
            .await
            .unwrap()
            .is_empty()
    );
    for _ in 0..2 {
        let error = client
            .tipranks_analysts(TipRanksAnalystsQuery::new())
            .await
            .unwrap_err();
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some("tipranks-analysts"));
        assert_eq!(error.status_code(), Some(200));
    }
    assert_eq!(executor.requests().len(), 3);
}
