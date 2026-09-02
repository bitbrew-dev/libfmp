mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        bulk::{
            BulkYearQuery, bulk_earnings_surprises, bulk_financial_ratios_ttm,
            bulk_key_metrics_ttm, bulk_stock_peers,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    query::Year,
    responses::bulk::{
        BulkEarningsSurprise, BulkFinancialRatiosTtm, BulkKeyMetricsTtm, BulkStockPeers,
    },
    transport::{HttpMethod, TransportResponse},
};

use support::{FixtureExecutor, json_fixture};

const KEY_METRICS: &[u8] = include_bytes!("fixtures/bulk_key_metrics_ttm.json");
const RATIOS: &[u8] = include_bytes!("fixtures/bulk_financial_ratios_ttm.json");
const PEERS: &[u8] = include_bytes!("fixtures/bulk_stock_peers.json");
const SURPRISES: &[u8] = include_bytes!("fixtures/bulk_earnings_surprises.json");

fn fixtures() -> [TransportResponse; 4] {
    [
        json_fixture(KEY_METRICS),
        json_fixture(RATIOS),
        json_fixture(PEERS),
        json_fixture(SURPRISES),
    ]
}

#[test]
fn descriptors_have_exact_get_ids_paths_response_types_and_only_documented_metadata() {
    assert_facts(&bulk_key_metrics_ttm(), "key-metrics-ttm-bulk");
    assert_facts(&bulk_financial_ratios_ttm(), "ratios-ttm-bulk");
    assert_facts(&bulk_stock_peers(), "peers-bulk");
    assert_facts(
        &bulk_earnings_surprises(BulkYearQuery::new(Year(2026))),
        "earnings-surprises-bulk",
    );
    assert_response_types(
        &bulk_key_metrics_ttm(),
        &bulk_financial_ratios_ttm(),
        &bulk_stock_peers(),
        &bulk_earnings_surprises(Year(2026).into()),
    );
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, Vec<R>>, path: &'static str) {
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

fn assert_response_types(
    _: &EndpointSpec<(), Vec<BulkKeyMetricsTtm>>,
    _: &EndpointSpec<(), Vec<BulkFinancialRatiosTtm>>,
    _: &EndpointSpec<(), Vec<BulkStockPeers>>,
    _: &EndpointSpec<BulkYearQuery, Vec<BulkEarningsSurprise>>,
) {
}

#[test]
fn earnings_surprise_year_is_required_exact_and_has_no_inferred_range_or_default() {
    for value in [0, 7, 2026, u32::MAX] {
        let query = BulkYearQuery::new(Year(value));
        assert_eq!(query.year(), Year(value));
        assert_eq!(bulk_earnings_surprises(query).query().year(), Year(value));
    }
}

#[tokio::test]
async fn custom_proxy_sends_one_exact_request_per_method_with_auth_headers_and_decodes_fixtures() {
    let executor = Arc::new(FixtureExecutor::new(fixtures()));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Bearer ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "bulk metrics")
        .executor(executor.clone())
        .build()
        .unwrap();

    let metrics = client.bulk_key_metrics_ttm().await.unwrap();
    let ratios = client.bulk_financial_ratios_ttm().await.unwrap();
    let peers = client.bulk_stock_peers().await.unwrap();
    let surprises = client.bulk_earnings_surprises(Year(2026)).await.unwrap();

    assert_eq!(metrics[0].ev_to_ebitda_ttm.as_str(), "-14.656106051669223");
    assert_eq!(
        ratios[0].net_income_per_ebt_ttm.as_str(),
        "0.8225101702576465"
    );
    assert_eq!(peers[0].peers, "600036.SS");
    assert_eq!(surprises[0].eps_actual.as_str(), "0.3631");

    let requests = executor.requests();
    assert_eq!(requests.len(), 4);
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Bearer proxy-secret"
            && request.expose_headers()["x-data-scope"] == "bulk metrics"
    }));
    assert_eq!(
        request_urls(&requests),
        [
            "https://proxy.example/router/gateway/stable/key-metrics-ttm-bulk",
            "https://proxy.example/router/gateway/stable/ratios-ttm-bulk",
            "https://proxy.example/router/gateway/stable/peers-bulk",
            "https://proxy.example/router/gateway/stable/earnings-surprises-bulk?year=2026",
        ]
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_preserve_paths_and_exact_query_shape() {
    for (authentication, unit_suffix, year_suffix, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "",
            "?year=2026",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "?apikey=query-secret",
            "?year=2026&apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new(fixtures()));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client.bulk_key_metrics_ttm().await.unwrap();
        client.bulk_financial_ratios_ttm().await.unwrap();
        client.bulk_stock_peers().await.unwrap();
        client.bulk_earnings_surprises(Year(2026)).await.unwrap();

        let requests = executor.requests();
        assert_eq!(requests.len(), 4);
        assert_eq!(
            request_urls(&requests),
            [
                format!(
                    "https://financialmodelingprep.com/stable/key-metrics-ttm-bulk{unit_suffix}"
                ),
                format!("https://financialmodelingprep.com/stable/ratios-ttm-bulk{unit_suffix}"),
                format!("https://financialmodelingprep.com/stable/peers-bulk{unit_suffix}"),
                format!(
                    "https://financialmodelingprep.com/stable/earnings-surprises-bulk{year_suffix}"
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
async fn empty_arrays_decode_and_malformed_roots_keep_all_four_endpoint_identities() {
    let executor = Arc::new(FixtureExecutor::new(
        std::iter::repeat_with(|| json_fixture(b"[]"))
            .take(4)
            .chain(std::iter::repeat_with(|| json_fixture(b"{}")).take(4)),
    ));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor.clone())
        .build()
        .unwrap();

    assert!(client.bulk_key_metrics_ttm().await.unwrap().is_empty());
    assert!(client.bulk_financial_ratios_ttm().await.unwrap().is_empty());
    assert!(client.bulk_stock_peers().await.unwrap().is_empty());
    assert!(
        client
            .bulk_earnings_surprises(Year(2026))
            .await
            .unwrap()
            .is_empty()
    );

    let errors = [
        client.bulk_key_metrics_ttm().await.unwrap_err(),
        client.bulk_financial_ratios_ttm().await.unwrap_err(),
        client.bulk_stock_peers().await.unwrap_err(),
        client
            .bulk_earnings_surprises(Year(2026))
            .await
            .unwrap_err(),
    ];
    for (error, id) in errors.iter().zip([
        "key-metrics-ttm-bulk",
        "ratios-ttm-bulk",
        "peers-bulk",
        "earnings-surprises-bulk",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(id));
        assert_eq!(error.status_code(), Some(200));
    }
    assert_eq!(executor.requests().len(), 8);
}

fn request_urls(requests: &[libfmp::transport::PreparedRequest]) -> Vec<String> {
    requests
        .iter()
        .map(|request| request.expose_url().as_str().to_owned())
        .collect()
}
