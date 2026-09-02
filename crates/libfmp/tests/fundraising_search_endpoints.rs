mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        fundraising::{
            OfferingSearchQuery, search_crowdfunding_offerings, search_regulation_d_offerings,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::fundraising::{CrowdfundingOfferingSearchResult, RegulationDOfferingSearchResult},
    transport::HttpMethod,
    types::{SearchTerm, StringValueError},
};

use support::{FixtureExecutor, json_fixture};

const CROWDFUNDING: &[u8] = include_bytes!("fixtures/crowdfunding_offerings_search.json");
const REGULATION_D: &[u8] = include_bytes!("fixtures/fundraising_search.json");

#[test]
fn descriptors_have_exact_identity_response_types_and_only_us_geography() {
    let name = SearchTerm::new("NJOY").unwrap();
    let owned = OfferingSearchQuery::from(name.clone());
    let borrowed = OfferingSearchQuery::from(&name);
    assert_eq!(owned.name(), &name);
    assert_eq!(borrowed.name(), &name);

    let crowdfunding = search_crowdfunding_offerings(owned);
    let regulation_d = search_regulation_d_offerings(borrowed);
    assert_facts(&crowdfunding, "crowdfunding-offerings-search");
    assert_facts(&regulation_d, "fundraising-search");
    assert_response_types(&crowdfunding, &regulation_d);
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, Vec<R>>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
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
    _: &EndpointSpec<OfferingSearchQuery, Vec<CrowdfundingOfferingSearchResult>>,
    _: &EndpointSpec<OfferingSearchQuery, Vec<RegulationDOfferingSearchResult>>,
) {
}

#[test]
fn required_name_uses_shared_validation() {
    assert_eq!(SearchTerm::new("").unwrap_err(), StringValueError::Empty);
    assert_eq!(
        SearchTerm::new(" \t ").unwrap_err(),
        StringValueError::Empty
    );
    assert_eq!(
        SearchTerm::new("NJOY\nINC").unwrap_err(),
        StringValueError::ControlCharacter
    );
}

#[tokio::test]
async fn custom_proxy_preserves_exact_paths_encoding_auth_headers_and_fixtures() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(CROWDFUNDING),
        json_fixture(REGULATION_D),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Bearer ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "fundraising searches")
        .executor(executor.clone())
        .build()
        .unwrap();
    let name = SearchTerm::new("NJOY / Class A").unwrap();

    let crowdfunding = client.search_crowdfunding_offerings(&name).await.unwrap();
    let regulation_d = client.search_regulation_d_offerings(name).await.unwrap();
    assert_eq!(crowdfunding[0].cik.as_str(), "0001912939");
    assert_eq!(regulation_d[0].cik.as_str(), "0001547416");

    let requests = executor.requests();
    assert_eq!(requests.len(), 2);
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Bearer proxy-secret"
            && request.expose_headers()["x-data-scope"] == "fundraising searches"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/crowdfunding-offerings-search?name=NJOY+%2F+Class+A",
            "https://proxy.example/router/gateway/stable/fundraising-search?name=NJOY+%2F+Class+A",
        ]
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_work_for_both_searches() {
    for (authentication, suffix, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([
            json_fixture(CROWDFUNDING),
            json_fixture(REGULATION_D),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .search_crowdfunding_offerings(SearchTerm::new("enotap").unwrap())
            .await
            .unwrap();
        client
            .search_regulation_d_offerings(SearchTerm::new("NJOY").unwrap())
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(
            requests[0].expose_url().as_str(),
            format!(
                "https://financialmodelingprep.com/stable/crowdfunding-offerings-search?name=enotap{suffix}"
            )
        );
        assert_eq!(
            requests[1].expose_url().as_str(),
            format!(
                "https://financialmodelingprep.com/stable/fundraising-search?name=NJOY{suffix}"
            )
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
async fn empty_arrays_decode_and_malformed_roots_preserve_both_endpoint_identities() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"[]"),
        json_fixture(b"[]"),
        json_fixture(b"{}"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();
    let name = || SearchTerm::new("NJOY").unwrap();

    assert!(
        client
            .search_crowdfunding_offerings(name())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .search_regulation_d_offerings(name())
            .await
            .unwrap()
            .is_empty()
    );

    let crowdfunding = client
        .search_crowdfunding_offerings(name())
        .await
        .unwrap_err();
    let regulation_d = client
        .search_regulation_d_offerings(name())
        .await
        .unwrap_err();
    assert_eq!(crowdfunding.category(), ErrorCategory::Decode);
    assert_eq!(
        crowdfunding.endpoint(),
        Some("crowdfunding-offerings-search")
    );
    assert_eq!(crowdfunding.status_code(), Some(200));
    assert_eq!(regulation_d.category(), ErrorCategory::Decode);
    assert_eq!(regulation_d.endpoint(), Some("fundraising-search"));
    assert_eq!(regulation_d.status_code(), Some(200));
}
