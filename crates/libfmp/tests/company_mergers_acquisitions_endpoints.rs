mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        company::{
            LatestMergersAcquisitionsQuery, SearchMergersAcquisitionsQuery,
            latest_mergers_acquisitions, search_mergers_acquisitions,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    transport::HttpMethod,
    types::{Limit, Page, SearchTerm},
};

use support::{FixtureExecutor, json_fixture};

const LATEST: &[u8] = include_bytes!("fixtures/company_mergers_acquisitions_latest.json");
const SEARCH: &[u8] = include_bytes!("fixtures/company_mergers_acquisitions_search.json");
const MULTIPLE: &[u8] = include_bytes!("fixtures/company_mergers_acquisitions_multiple.json");
const UNKNOWN: &[u8] = include_bytes!("fixtures/company_mergers_acquisitions_unknown.json");
const EMPTY: &[u8] = include_bytes!("fixtures/company_empty.json");

#[test]
fn descriptors_use_exact_paths_queries_us_geography_and_response_row_metadata() {
    let latest_query = LatestMergersAcquisitionsQuery::new()
        .with_page(Page(0))
        .with_limit(Limit(1_001));
    let name = SearchTerm::new("  Apple, Inc. / Class A  ").unwrap();
    let search_query = SearchMergersAcquisitionsQuery::new(name.clone());

    assert_eq!(latest_query.page(), Some(Page(0)));
    assert_eq!(latest_query.limit(), Some(Limit(1_001)));
    assert_eq!(search_query.name(), &name);

    let latest = latest_mergers_acquisitions(latest_query);
    let search = search_mergers_acquisitions(search_query);
    assert_eq!(facts(&latest), "mergers-acquisitions-latest");
    assert_eq!(facts(&search), "mergers-acquisitions-search");

    let latest_bounds = latest.metadata().bounds();
    assert_eq!(latest_bounds.limit(), None);
    assert_eq!(latest_bounds.page(), None);
    assert!(latest_bounds.accepts_limit(Limit(u32::MAX)));
    assert!(latest_bounds.accepts_page(Page(u32::MAX)));
    assert!(latest_bounds.accepts_response_rows(1_000));
    assert!(!latest_bounds.accepts_response_rows(1_001));
    assert_eq!(search.metadata().bounds(), EndpointBounds::new());
}

fn facts<Q, R>(endpoint: &EndpointSpec<Q, R>) -> &'static str {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), endpoint.relative_path());
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::UsOnly
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
    endpoint.id()
}

#[tokio::test]
async fn proxy_client_preserves_omission_order_zero_above_bound_and_name_encoding() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(LATEST),
        json_fixture(LATEST),
        json_fixture(SEARCH),
    ]));
    let client = proxy_client(executor.clone());

    let latest = client
        .latest_mergers_acquisitions(LatestMergersAcquisitionsQuery::new())
        .await
        .unwrap();
    assert_eq!(latest[0].cik.as_str(), "0002009312");
    assert_eq!(latest[0].targeted_cik.as_str(), "0002009312");
    assert_eq!(
        latest[0].targeted_company_name,
        "Aureus Greenway Holdings, Inc."
    );
    assert_eq!(
        latest[0].link,
        "https://www.sec.gov/Archives/edgar/data/2009312/000149315226035181/forms-4.htm"
    );

    client
        .latest_mergers_acquisitions(
            LatestMergersAcquisitionsQuery::new()
                .with_page(Page(0))
                .with_limit(Limit(1_001)),
        )
        .await
        .unwrap();
    let search = client
        .search_mergers_acquisitions(SearchTerm::new("  Apple, Inc. / Class A  ").unwrap())
        .await
        .unwrap();
    assert_eq!(search[0].cik.as_str(), "0000022701");
    assert_eq!(
        search[0].targeted_company_name,
        "Communications Systems, Inc."
    );

    let requests = executor.requests();
    let urls = requests
        .iter()
        .map(|request| request.expose_url().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        urls,
        [
            "https://proxy.example/router/stable/mergers-acquisitions-latest",
            "https://proxy.example/router/stable/mergers-acquisitions-latest?page=0&limit=1001",
            "https://proxy.example/router/stable/mergers-acquisitions-search?name=++Apple%2C+Inc.+%2F+Class+A++",
        ]
    );
    assert!(!urls[0].contains('?'));
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
}

#[tokio::test]
async fn merger_acquisition_arrays_preserve_multiple_empty_and_unknown_field_payloads() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(MULTIPLE),
        json_fixture(UNKNOWN),
        json_fixture(EMPTY),
        json_fixture(EMPTY),
    ]));
    let client = proxy_client(executor);

    let multiple = client
        .latest_mergers_acquisitions(LatestMergersAcquisitionsQuery::new())
        .await
        .unwrap();
    assert_eq!(multiple.len(), 2);
    assert_eq!(multiple[1].symbol.as_str(), "PEGY");
    assert_eq!(multiple[1].targeted_symbol.as_str(), "JCS");
    assert_eq!(multiple[1].transaction_date.to_string(), "2021-11-12");
    assert_eq!(multiple[1].accepted_date.to_string(), "2021-11-12 09:54:22");

    let unknown = client
        .search_mergers_acquisitions(SearchTerm::new("Apple").unwrap())
        .await
        .unwrap();
    assert_eq!(unknown.len(), 1);
    assert_eq!(unknown[0].company_name, "Aureus Greenway Holdings Inc");

    assert!(
        client
            .latest_mergers_acquisitions(LatestMergersAcquisitionsQuery::new())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .search_mergers_acquisitions(SearchTerm::new("Apple").unwrap())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_use_the_same_merger_acquisition_contracts() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/mergers-acquisitions-latest?page=0",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/mergers-acquisitions-search?name=Apple&apikey=query-secret",
            None,
        ),
    ] {
        let response = if expected_header.is_some() {
            LATEST
        } else {
            SEARCH
        };
        let executor = Arc::new(FixtureExecutor::new([json_fixture(response)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        if expected_header.is_some() {
            client
                .latest_mergers_acquisitions(
                    LatestMergersAcquisitionsQuery::new().with_page(Page(0)),
                )
                .await
                .unwrap();
        } else {
            client
                .search_mergers_acquisitions(SearchTerm::new("Apple").unwrap())
                .await
                .unwrap();
        }

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
