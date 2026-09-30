mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        company::{
            CompanyNotesQuery, ProfileByCikQuery, ProfileQuery, StockPeersQuery, company_notes,
            profile, profile_by_cik, stock_peers,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    transport::HttpMethod,
    types::{Cik, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const PROFILE: &[u8] = include_bytes!("fixtures/company_profile.json");
const NOTE: &[u8] = include_bytes!("fixtures/company_note.json");
const PEER: &[u8] = include_bytes!("fixtures/stock_peer.json");
const EMPTY: &[u8] = include_bytes!("fixtures/company_empty.json");
const MULTIPLE: &[u8] = include_bytes!("fixtures/company_profile_multiple.json");
const UNKNOWN: &[u8] = include_bytes!("fixtures/company_profile_unknown.json");

#[test]
fn descriptors_use_exact_paths_queries_response_shapes_and_sourced_geography() {
    let profile_query = ProfileQuery::new(Ticker::new("AAPL").unwrap());
    let cik_query = ProfileByCikQuery::new(Cik::new("0000320193").unwrap());
    let notes_query = CompanyNotesQuery::new(Ticker::new("AAPL").unwrap());
    let peers_query = StockPeersQuery::new(Ticker::new("AAPL").unwrap());

    assert_eq!(profile_query.symbol().as_str(), "AAPL");
    assert_eq!(cik_query.cik().as_str(), "0000320193");
    assert_eq!(notes_query.symbol().as_str(), "AAPL");
    assert_eq!(peers_query.symbol().as_str(), "AAPL");

    assert_eq!(
        facts(&profile(profile_query)),
        ("profile", GeographicAvailability::Worldwide)
    );
    assert_eq!(
        facts(&profile_by_cik(cik_query)),
        ("profile-cik", GeographicAvailability::UsOnly)
    );
    assert_eq!(
        facts(&company_notes(notes_query)),
        ("company-notes", GeographicAvailability::UsOnly)
    );
    assert_eq!(
        facts(&stock_peers(peers_query)),
        ("stock-peers", GeographicAvailability::Worldwide)
    );
}

fn facts<Q, R>(endpoint: &EndpointSpec<Q, R>) -> (&'static str, GeographicAvailability) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), endpoint.relative_path());
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
    (endpoint.id(), endpoint.metadata().geography())
}

#[tokio::test]
async fn custom_proxy_client_executes_all_four_paths_with_exact_required_queries() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(PROFILE),
        json_fixture(PROFILE),
        json_fixture(NOTE),
        json_fixture(PEER),
    ]));
    let client = proxy_client(executor.clone());

    assert_eq!(
        client
            .profile(Ticker::new("BRK.B / Class A").unwrap())
            .await
            .unwrap()[0]
            .cik
            .as_ref()
            .unwrap()
            .as_str(),
        "0000320193"
    );
    assert_eq!(
        client
            .profile_by_cik(Cik::new("0000320193").unwrap())
            .await
            .unwrap()[0]
            .full_time_employees
            .as_ref()
            .unwrap()
            .as_str(),
        "166000"
    );
    assert_eq!(
        client
            .company_notes(CompanyNotesQuery::new(Ticker::new("AAPL").unwrap()))
            .await
            .unwrap()[0]
            .title,
        "0.000% Notes due 2025"
    );
    assert_eq!(
        client
            .stock_peers(StockPeersQuery::new(Ticker::new("^VIX").unwrap()))
            .await
            .unwrap()[0]
            .market_cap,
        4_040_168_831_718.0
    );

    let requests = executor.requests();
    let urls = requests
        .iter()
        .map(|request| request.expose_url().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        urls,
        [
            "https://proxy.example/router/stable/profile?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/stable/profile-cik?cik=0000320193",
            "https://proxy.example/router/stable/company-notes?symbol=AAPL",
            "https://proxy.example/router/stable/stock-peers?symbol=%5EVIX",
        ]
    );
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
}

#[tokio::test]
async fn profile_client_preserves_empty_multiple_and_unknown_field_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(EMPTY),
        json_fixture(MULTIPLE),
        json_fixture(UNKNOWN),
    ]));
    let client = proxy_client(executor);
    let query = || ProfileQuery::new(Ticker::new("AAPL").unwrap());

    let empty = client.profile(query()).await.unwrap();
    let multiple = client.profile(query()).await.unwrap();
    let unknown = client.profile(query()).await.unwrap();

    assert!(empty.is_empty());
    assert_eq!(multiple.len(), 2);
    assert_eq!(multiple[0].market_cap, 9_007_199_254_740_992.0);
    assert_eq!(multiple[0].volume, u64::MAX as f64);
    assert_eq!(unknown.len(), 1);
    assert_eq!(unknown[0].cik.as_ref().unwrap().as_str(), "0000320193");
}

#[tokio::test]
async fn every_company_client_method_preserves_empty_arrays() {
    let executor = Arc::new(FixtureExecutor::new(
        std::iter::repeat_with(|| json_fixture(EMPTY)).take(4),
    ));
    let client = proxy_client(executor);

    assert!(
        client
            .profile(Ticker::new("AAPL").unwrap())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .profile_by_cik(Cik::new("320193").unwrap())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .company_notes(Ticker::new("AAPL").unwrap())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .stock_peers(Ticker::new("AAPL").unwrap())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn direct_fmp_auth_uses_the_same_profile_contract() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/profile?symbol=AAPL",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/profile?symbol=AAPL&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(PROFILE)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client.profile(Ticker::new("AAPL").unwrap()).await.unwrap();

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
