mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        bulk::{
            BulkPartQuery, bulk_company_profiles, bulk_dcf_valuations, bulk_etf_holdings,
            bulk_financial_scores, bulk_price_target_summaries, bulk_stock_ratings,
            bulk_upgrades_downgrades_consensus,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::{
        bulk::{
            BulkDcfValuation, BulkEtfHolding, BulkFinancialScore, BulkPriceTargetSummary,
            BulkStockRating, BulkUpgradesDowngradesConsensus,
        },
        company::CompanyProfile,
    },
    transport::{HttpMethod, TransportResponse},
    types::{BulkPart, StringValueError},
};

use support::{FixtureExecutor, fixture_response, json_fixture};

const PROFILE: &[u8] = include_bytes!("fixtures/bulk_company_profiles.csv");
const RATING: &[u8] = include_bytes!("fixtures/bulk_stock_ratings.csv");
const DCF: &[u8] = include_bytes!("fixtures/bulk_dcf_valuations.csv");
const SCORES: &[u8] = include_bytes!("fixtures/bulk_financial_scores.csv");
const TARGET: &[u8] = include_bytes!("fixtures/bulk_price_target_summaries.csv");
const ETF: &[u8] = include_bytes!("fixtures/bulk_etf_holdings.csv");
const CONSENSUS: &[u8] = include_bytes!("fixtures/bulk_upgrades_downgrades_consensus.csv");

fn csv_fixture(body: &'static [u8]) -> TransportResponse {
    fixture_response(Some("text/csv"), body)
}

fn fixtures() -> [TransportResponse; 7] {
    [
        csv_fixture(PROFILE),
        csv_fixture(RATING),
        csv_fixture(DCF),
        csv_fixture(SCORES),
        csv_fixture(TARGET),
        csv_fixture(ETF),
        csv_fixture(CONSENSUS),
    ]
}

#[test]
fn descriptors_have_exact_get_paths_response_types_and_only_documented_geography() {
    let part = BulkPart::new("0").unwrap();
    assert_facts(
        &bulk_company_profiles((&part).into()),
        "profile-bulk",
        GeographicAvailability::Worldwide,
    );
    assert_facts(
        &bulk_stock_ratings(),
        "rating-bulk",
        GeographicAvailability::Worldwide,
    );
    assert_facts(
        &bulk_dcf_valuations(),
        "dcf-bulk",
        GeographicAvailability::Worldwide,
    );
    assert_facts(
        &bulk_financial_scores(),
        "scores-bulk",
        GeographicAvailability::Worldwide,
    );
    assert_facts(
        &bulk_price_target_summaries(),
        "price-target-summary-bulk",
        GeographicAvailability::UsOnly,
    );
    assert_facts(
        &bulk_etf_holdings((&part).into()),
        "etf-holder-bulk",
        GeographicAvailability::Worldwide,
    );
    assert_facts(
        &bulk_upgrades_downgrades_consensus(),
        "upgrades-downgrades-consensus-bulk",
        GeographicAvailability::Worldwide,
    );

    assert_response_types(
        &bulk_company_profiles((&part).into()),
        &bulk_stock_ratings(),
        &bulk_dcf_valuations(),
        &bulk_financial_scores(),
        &bulk_price_target_summaries(),
        &bulk_etf_holdings((&part).into()),
        &bulk_upgrades_downgrades_consensus(),
    );
}

fn assert_facts<Q, R>(
    endpoint: &EndpointSpec<Q, Vec<R>>,
    path: &'static str,
    geography: GeographicAvailability,
) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(endpoint.metadata().geography(), geography);
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), EndpointBounds::new());
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[allow(clippy::too_many_arguments)]
fn assert_response_types(
    _: &EndpointSpec<BulkPartQuery, Vec<CompanyProfile>>,
    _: &EndpointSpec<(), Vec<BulkStockRating>>,
    _: &EndpointSpec<(), Vec<BulkDcfValuation>>,
    _: &EndpointSpec<(), Vec<BulkFinancialScore>>,
    _: &EndpointSpec<(), Vec<BulkPriceTargetSummary>>,
    _: &EndpointSpec<BulkPartQuery, Vec<BulkEtfHolding>>,
    _: &EndpointSpec<(), Vec<BulkUpgradesDowngradesConsensus>>,
) {
}

#[test]
fn bulk_part_is_open_validated_and_representation_preserving_without_numeric_inference() {
    for invalid in ["", "  "] {
        assert_eq!(BulkPart::new(invalid).unwrap_err(), StringValueError::Empty);
    }
    assert_eq!(
        BulkPart::new("part\n1").unwrap_err(),
        StringValueError::ControlCharacter
    );

    for value in ["0", "1", "0001", "segment A/7", "-1", "999999999999"] {
        let part = BulkPart::new(value).unwrap();
        assert_eq!(part.as_str(), value);
        assert_eq!(part.to_string(), value);
        assert_eq!(
            serde_json::to_string(&part).unwrap(),
            format!("\"{value}\"")
        );
    }
}

#[tokio::test]
async fn custom_proxy_sends_one_exact_request_per_method_with_auth_headers_and_fixtures() {
    let executor = Arc::new(FixtureExecutor::new(fixtures()));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Bearer ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "bulk snapshots")
        .executor(executor.clone())
        .build()
        .unwrap();

    let profile = client
        .bulk_company_profiles(BulkPart::new("segment 0/alpha").unwrap())
        .await
        .unwrap();
    let rating = client.bulk_stock_ratings().await.unwrap();
    let dcf = client.bulk_dcf_valuations().await.unwrap();
    let scores = client.bulk_financial_scores().await.unwrap();
    let target = client.bulk_price_target_summaries().await.unwrap();
    let etf = client
        .bulk_etf_holdings(BulkPart::new("part 1/beta").unwrap())
        .await
        .unwrap();
    let consensus = client.bulk_upgrades_downgrades_consensus().await.unwrap();

    assert_eq!(profile[1].symbol.as_str(), "AMAT");
    assert_eq!(rating[1].rating, "C+");
    assert_eq!(dcf[1].stock_price.as_str(), "2.39");
    assert_eq!(scores[0].reported_currency.as_str(), "CNY");
    assert_eq!(target[1].symbol.as_str(), "AA");
    assert_eq!(etf[1].asset.as_str(), "3665.TW");
    assert_eq!(consensus[1].consensus, "Hold");

    let requests = executor.requests();
    assert_eq!(requests.len(), 7);
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Bearer proxy-secret"
            && request.expose_headers()["x-data-scope"] == "bulk snapshots"
    }));
    assert_eq!(
        request_urls(&requests),
        [
            "https://proxy.example/router/gateway/stable/profile-bulk?part=segment+0%2Falpha",
            "https://proxy.example/router/gateway/stable/rating-bulk",
            "https://proxy.example/router/gateway/stable/dcf-bulk",
            "https://proxy.example/router/gateway/stable/scores-bulk",
            "https://proxy.example/router/gateway/stable/price-target-summary-bulk",
            "https://proxy.example/router/gateway/stable/etf-holder-bulk?part=part+1%2Fbeta",
            "https://proxy.example/router/gateway/stable/upgrades-downgrades-consensus-bulk",
        ]
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_preserve_paths_and_exact_query_shape() {
    for (authentication, suffix, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new(fixtures()));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .bulk_company_profiles(BulkPart::new("0").unwrap())
            .await
            .unwrap();
        client.bulk_stock_ratings().await.unwrap();
        client.bulk_dcf_valuations().await.unwrap();
        client.bulk_financial_scores().await.unwrap();
        client.bulk_price_target_summaries().await.unwrap();
        client
            .bulk_etf_holdings(BulkPart::new("1").unwrap())
            .await
            .unwrap();
        client.bulk_upgrades_downgrades_consensus().await.unwrap();

        let query_suffix = if suffix.is_empty() {
            String::new()
        } else {
            format!("&{suffix}")
        };
        let unit_suffix = if suffix.is_empty() {
            String::new()
        } else {
            format!("?{suffix}")
        };
        let requests = executor.requests();
        assert_eq!(
            request_urls(&requests),
            [
                format!(
                    "https://financialmodelingprep.com/stable/profile-bulk?part=0{query_suffix}"
                ),
                format!("https://financialmodelingprep.com/stable/rating-bulk{unit_suffix}"),
                format!("https://financialmodelingprep.com/stable/dcf-bulk{unit_suffix}"),
                format!("https://financialmodelingprep.com/stable/scores-bulk{unit_suffix}"),
                format!(
                    "https://financialmodelingprep.com/stable/price-target-summary-bulk{unit_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/etf-holder-bulk?part=1{query_suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/upgrades-downgrades-consensus-bulk{unit_suffix}"
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
async fn empty_bodies_decode_and_json_bodies_keep_all_endpoint_identities() {
    let executor = Arc::new(FixtureExecutor::new(
        std::iter::repeat_with(|| csv_fixture(b""))
            .take(7)
            .chain(std::iter::repeat_with(|| json_fixture(b"[]")).take(7)),
    ));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();

    assert!(
        client
            .bulk_company_profiles(BulkPart::new("0").unwrap())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(client.bulk_stock_ratings().await.unwrap().is_empty());
    assert!(client.bulk_dcf_valuations().await.unwrap().is_empty());
    assert!(client.bulk_financial_scores().await.unwrap().is_empty());
    assert!(
        client
            .bulk_price_target_summaries()
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .bulk_etf_holdings(BulkPart::new("1").unwrap())
            .await
            .unwrap()
            .is_empty()
    );
    assert!(
        client
            .bulk_upgrades_downgrades_consensus()
            .await
            .unwrap()
            .is_empty()
    );

    let errors = [
        client
            .bulk_company_profiles(BulkPart::new("0").unwrap())
            .await
            .unwrap_err(),
        client.bulk_stock_ratings().await.unwrap_err(),
        client.bulk_dcf_valuations().await.unwrap_err(),
        client.bulk_financial_scores().await.unwrap_err(),
        client.bulk_price_target_summaries().await.unwrap_err(),
        client
            .bulk_etf_holdings(BulkPart::new("1").unwrap())
            .await
            .unwrap_err(),
        client
            .bulk_upgrades_downgrades_consensus()
            .await
            .unwrap_err(),
    ];
    for (error, id) in errors.iter().zip([
        "profile-bulk",
        "rating-bulk",
        "dcf-bulk",
        "scores-bulk",
        "price-target-summary-bulk",
        "etf-holder-bulk",
        "upgrades-downgrades-consensus-bulk",
    ]) {
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
