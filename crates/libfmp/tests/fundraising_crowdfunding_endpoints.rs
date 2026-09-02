mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        fundraising::{
            LatestCrowdfundingOfferingsQuery, OfferingByCikQuery, crowdfunding_offerings_by_cik,
            latest_crowdfunding_offerings,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::fundraising::CrowdfundingOffering,
    transport::HttpMethod,
    types::{Cik, Limit, Page, StringValueError},
};

use support::{FixtureExecutor, json_fixture};

const LATEST: &[u8] = include_bytes!("fixtures/crowdfunding_offerings_latest.json");
const BY_CIK: &[u8] = include_bytes!("fixtures/crowdfunding_offerings_by_cik.json");

#[test]
fn descriptors_have_exact_get_identity_shared_rows_and_only_us_geography() {
    let latest = latest_crowdfunding_offerings(LatestCrowdfundingOfferingsQuery::new());
    let by_cik =
        crowdfunding_offerings_by_cik(OfferingByCikQuery::new(Cik::new("0001916078").unwrap()));
    assert_facts(&latest, "crowdfunding-offerings-latest");
    assert_facts(&by_cik, "crowdfunding-offerings");
    assert_response_types(&latest, &by_cik);
}

fn assert_facts<Q>(endpoint: &EndpointSpec<Q, Vec<CrowdfundingOffering>>, path: &'static str) {
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
    _: &EndpointSpec<LatestCrowdfundingOfferingsQuery, Vec<CrowdfundingOffering>>,
    _: &EndpointSpec<OfferingByCikQuery, Vec<CrowdfundingOffering>>,
) {
}

#[test]
fn queries_preserve_independent_omission_full_domains_exact_order_and_required_cik() {
    let empty = LatestCrowdfundingOfferingsQuery::new();
    assert_eq!(empty.page(), None);
    assert_eq!(empty.limit(), None);
    let complete = empty.with_page(Page(0)).with_limit(Limit(u32::MAX));
    assert_eq!(complete.page(), Some(Page(0)));
    assert_eq!(complete.limit(), Some(Limit(u32::MAX)));

    let cik = Cik::new("0001916078").unwrap();
    let owned = OfferingByCikQuery::from(cik.clone());
    let borrowed = OfferingByCikQuery::from(&cik);
    assert_eq!(owned.cik(), &cik);
    assert_eq!(borrowed.cik(), &cik);
    assert_eq!(Cik::new("").unwrap_err(), StringValueError::Empty);
    assert_eq!(Cik::new("\n").unwrap_err(), StringValueError::Empty);
}

#[tokio::test]
async fn proxy_preserves_paths_queries_custom_auth_headers_and_exact_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(LATEST),
        json_fixture(BY_CIK),
        json_fixture(LATEST),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Bearer ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "crowdfunding offerings")
        .executor(executor.clone())
        .build()
        .unwrap();

    let latest = client
        .latest_crowdfunding_offerings(
            LatestCrowdfundingOfferingsQuery::new()
                .with_page(Page(0))
                .with_limit(Limit(100)),
        )
        .await
        .unwrap();
    let by_cik = client
        .crowdfunding_offerings_by_cik(Cik::new("0001916078").unwrap())
        .await
        .unwrap();
    client
        .latest_crowdfunding_offerings(LatestCrowdfundingOfferingsQuery::new().with_limit(Limit(0)))
        .await
        .unwrap();

    assert_eq!(latest[0].cik.as_str(), "0001621902");
    assert_eq!(by_cik[0].cik.as_str(), "0001916078");
    assert_eq!(
        serde_json::to_value(&latest).unwrap(),
        serde_json::from_slice::<serde_json::Value>(LATEST).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&by_cik).unwrap(),
        serde_json::from_slice::<serde_json::Value>(BY_CIK).unwrap()
    );

    let requests = executor.requests();
    assert_eq!(requests.len(), 3);
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Bearer proxy-secret"
            && request.expose_headers()["x-data-scope"] == "crowdfunding offerings"
    }));
    assert_eq!(
        request_urls(&requests),
        [
            "https://proxy.example/router/gateway/stable/crowdfunding-offerings-latest?page=0&limit=100",
            "https://proxy.example/router/gateway/stable/crowdfunding-offerings?cik=0001916078",
            "https://proxy.example/router/gateway/stable/crowdfunding-offerings-latest?limit=0",
        ]
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_preserve_both_contracts() {
    for (authentication, expected_url, expected_header, latest) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/crowdfunding-offerings-latest",
            Some("header-secret"),
            true,
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/crowdfunding-offerings?cik=0001916078&apikey=query-secret",
            None,
            false,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(if latest {
            LATEST
        } else {
            BY_CIK
        })]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();
        if latest {
            client
                .latest_crowdfunding_offerings(LatestCrowdfundingOfferingsQuery::new())
                .await
                .unwrap();
        } else {
            client
                .crowdfunding_offerings_by_cik(Cik::new("0001916078").unwrap())
                .await
                .unwrap();
        }
        let requests = executor.requests();
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some(value) => assert_eq!(requests[0].expose_headers()["apikey"], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}

#[tokio::test]
async fn empty_arrays_decode_and_malformed_roots_preserve_endpoint_identities() {
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

    assert!(
        client
            .latest_crowdfunding_offerings(LatestCrowdfundingOfferingsQuery::new())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .crowdfunding_offerings_by_cik(Cik::new("0001916078").unwrap())
            .await
            .unwrap()
            .is_empty()
    );

    let errors = [
        client
            .latest_crowdfunding_offerings(LatestCrowdfundingOfferingsQuery::new())
            .await
            .unwrap_err(),
        client
            .crowdfunding_offerings_by_cik(Cik::new("0001916078").unwrap())
            .await
            .unwrap_err(),
    ];
    for (error, id) in errors
        .iter()
        .zip(["crowdfunding-offerings-latest", "crowdfunding-offerings"])
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
