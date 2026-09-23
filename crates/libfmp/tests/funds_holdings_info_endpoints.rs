mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    codecs::IsoTimestamp,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        funds::{EtfHoldingsQuery, EtfInfoQuery, etf_holdings, etf_info},
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::funds::{EtfFundHolding, EtfFundInfo},
    transport::HttpMethod,
    types::{ApiDateTime, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const HOLDINGS: &[u8] = include_bytes!("fixtures/etf_fund_holdings.json");
const INFO: &[u8] = include_bytes!("fixtures/etf_fund_info.json");

#[test]
fn descriptors_use_exact_paths_bare_types_and_only_documented_metadata() {
    let holdings = etf_holdings(EtfHoldingsQuery::new(Ticker::new("SPY").unwrap()));
    let info = etf_info(EtfInfoQuery::new(Ticker::new("SPY").unwrap()));

    assert_facts(&holdings, "etf/holdings");
    assert_facts(&info, "etf/info");
    assert_response_types(&holdings, &info);
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
    _: &EndpointSpec<EtfHoldingsQuery, Vec<EtfFundHolding>>,
    _: &EndpointSpec<EtfInfoQuery, Vec<EtfFundInfo>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_exact_queries_headers_and_fixture_identity() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(HOLDINGS),
        json_fixture(INFO),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "fund holdings and info")
        .executor(executor.clone())
        .build()
        .unwrap();

    let holdings = client
        .etf_holdings(EtfHoldingsQuery::new(
            Ticker::new("BRK.B / Class A").unwrap(),
        ))
        .await
        .unwrap();
    let info = client
        .etf_info(EtfInfoQuery::new(Ticker::new("SPY").unwrap()))
        .await
        .unwrap();

    assert_eq!(holdings.len(), 1);
    assert_eq!(holdings[0].market_value, 61_679_458_958.0);
    assert_eq!(holdings[0].shares_number, 181_418_073);
    assert_eq!(
        holdings[0].updated_at,
        ApiDateTime::parse("2026-07-30 08:07:21").unwrap()
    );
    assert_eq!(info.len(), 1);
    assert_eq!(info[0].assets_under_management, 777_349_860_000);
    assert_eq!(info[0].avg_volume, 52_093_933.0);
    assert_eq!(info[0].sectors_list.len(), 3);
    assert_eq!(info[0].sectors_list[1].industry.as_str(), "Cash & Others");
    assert_eq!(info[0].sectors_list[1].exposure, 0.30489782336177595);
    assert_eq!(
        info[0].updated_at,
        IsoTimestamp::from_str("2026-07-30T16:00:20.049Z").unwrap()
    );

    let requests = executor.requests();
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "fund holdings and info"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/etf/holdings?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/gateway/stable/etf/info?symbol=SPY",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_both_exact_urls() {
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
            json_fixture(HOLDINGS),
            json_fixture(INFO),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .etf_holdings(EtfHoldingsQuery::new(Ticker::new("SPY").unwrap()))
            .await
            .unwrap();
        client
            .etf_info(EtfInfoQuery::new(Ticker::new("SPY").unwrap()))
            .await
            .unwrap();

        let requests = executor.requests();
        assert_eq!(
            requests
                .iter()
                .map(|request| request.expose_url().as_str())
                .collect::<Vec<_>>(),
            [
                format!("https://financialmodelingprep.com/stable/etf/holdings?symbol=SPY{suffix}"),
                format!("https://financialmodelingprep.com/stable/etf/info?symbol=SPY{suffix}"),
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
async fn malformed_non_arrays_keep_each_endpoint_identity() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"{}"),
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();

    let errors = [
        client
            .etf_holdings(EtfHoldingsQuery::new(Ticker::new("SPY").unwrap()))
            .await
            .unwrap_err(),
        client
            .etf_info(EtfInfoQuery::new(Ticker::new("SPY").unwrap()))
            .await
            .unwrap_err(),
    ];
    for (error, endpoint) in errors.iter().zip(["etf/holdings", "etf/info"]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}
