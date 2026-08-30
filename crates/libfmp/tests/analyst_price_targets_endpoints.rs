mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        analyst::{
            PriceTargetConsensusQuery, PriceTargetSummaryQuery, price_target_consensus,
            price_target_summary,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::analyst::{PriceTargetConsensus, PriceTargetSummary},
    transport::HttpMethod,
    types::Ticker,
};

use support::{FixtureExecutor, json_fixture};

const SUMMARY: &[u8] = include_bytes!("fixtures/price_target_summary.json");
const CONSENSUS: &[u8] = include_bytes!("fixtures/price_target_consensus.json");
const PUBLISHERS: &str = "[\"StreetInsider\",\"TheFly\",\"Benzinga\",\"Pulse 2.0\",\"TipRanks Contributor\",\"MarketWatch\",\"Investing\",\"Barrons\",\"Investor's Business Daily\"]";

#[test]
fn descriptors_use_exact_paths_bare_rows_and_only_documented_metadata() {
    let summary = price_target_summary(PriceTargetSummaryQuery::new(Ticker::new("AAPL").unwrap()));
    assert_facts(&summary, "price-target-summary");

    let consensus =
        price_target_consensus(PriceTargetConsensusQuery::new(Ticker::new("AAPL").unwrap()));
    assert_facts(&consensus, "price-target-consensus");
    assert_response_types(&summary, &consensus);
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
    _: &EndpointSpec<PriceTargetSummaryQuery, Vec<PriceTargetSummary>>,
    _: &EndpointSpec<PriceTargetConsensusQuery, Vec<PriceTargetConsensus>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_exact_queries_headers_raw_publishers_and_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(SUMMARY),
        json_fixture(CONSENSUS),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "price-targets")
        .executor(executor.clone())
        .build()
        .unwrap();

    let summary = client
        .price_target_summary(PriceTargetSummaryQuery::new(
            Ticker::new("BRK.B / Class A").unwrap(),
        ))
        .await
        .unwrap();
    let consensus = client
        .price_target_consensus(PriceTargetConsensusQuery::new(Ticker::new("AAPL").unwrap()))
        .await
        .unwrap();

    assert_eq!(summary.len(), 1);
    assert_eq!(summary[0].publishers, PUBLISHERS);
    assert_eq!(summary[0].all_time_count, 254);
    assert_eq!(summary[0].last_month_avg_price_target, 333.75);
    assert_eq!(consensus.len(), 1);
    assert_eq!(consensus[0].target_high, 400.0);
    assert_eq!(consensus[0].target_consensus, 337.67);
    assert_eq!(consensus[0].target_median, 340.0);

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "price-targets"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/price-target-summary?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/gateway/stable/price-target-consensus?symbol=AAPL",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_exact_urls_and_response_identity() {
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
            json_fixture(SUMMARY),
            json_fixture(CONSENSUS),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .price_target_summary(PriceTargetSummaryQuery::new(Ticker::new("AAPL").unwrap()))
            .await
            .unwrap();
        client
            .price_target_consensus(PriceTargetConsensusQuery::new(Ticker::new("AAPL").unwrap()))
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(
            requests
                .iter()
                .map(|request| request.expose_url().as_str())
                .collect::<Vec<_>>(),
            [
                format!(
                    "https://financialmodelingprep.com/stable/price-target-summary?symbol=AAPL{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/price-target-consensus?symbol=AAPL{suffix}"
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
async fn malformed_non_array_responses_keep_each_endpoint_identity() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"{}"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();

    let summary_error = client
        .price_target_summary(PriceTargetSummaryQuery::new(Ticker::new("AAPL").unwrap()))
        .await
        .unwrap_err();
    assert_eq!(summary_error.category(), ErrorCategory::Decode);
    assert_eq!(summary_error.endpoint(), Some("price-target-summary"));
    assert_eq!(summary_error.status_code(), Some(200));

    let consensus_error = client
        .price_target_consensus(PriceTargetConsensusQuery::new(Ticker::new("AAPL").unwrap()))
        .await
        .unwrap_err();
    assert_eq!(consensus_error.category(), ErrorCategory::Decode);
    assert_eq!(consensus_error.endpoint(), Some("price-target-consensus"));
    assert_eq!(consensus_error.status_code(), Some(200));
}
