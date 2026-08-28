mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        statements::{KeyMetricsQuery, KeyMetricsTtmQuery, key_metrics, key_metrics_ttm},
    },
    query::{FiscalPeriod, RetrievalFrequency, StatementPeriod},
    transport::HttpMethod,
    types::{Limit, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const HISTORICAL: &[u8] = include_bytes!("fixtures/key_metrics.json");
const TTM: &[u8] = include_bytes!("fixtures/key_metrics_ttm.json");

#[test]
fn descriptors_use_exact_paths_shapes_and_only_documented_metadata() {
    let symbol = Ticker::new("AAPL").unwrap();
    let historical_query = KeyMetricsQuery::new(symbol.clone())
        .with_limit(Limit(1_000))
        .with_period(RetrievalFrequency::Annual);
    let ttm_query: KeyMetricsTtmQuery = (&symbol).into();

    assert_eq!(historical_query.symbol(), &symbol);
    assert_eq!(historical_query.limit(), Some(Limit(1_000)));
    assert_eq!(
        historical_query.period(),
        Some(RetrievalFrequency::Annual.into())
    );
    assert_eq!(ttm_query.symbol(), &symbol);

    assert_facts(
        &key_metrics(historical_query),
        "key-metrics",
        EndpointBounds::new().with_response_rows(1_000),
    );
    assert_facts(
        &key_metrics_ttm(ttm_query),
        "key-metrics-ttm",
        EndpointBounds::new(),
    );
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str, bounds: EndpointBounds) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(endpoint.metadata().bounds(), bounds);
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn proxy_preserves_omission_zero_order_all_periods_custom_auth_and_headers() {
    let executor = Arc::new(FixtureExecutor::new((0..10).map(|index| {
        if index == 9 {
            json_fixture(TTM)
        } else {
            json_fixture(HISTORICAL)
        }
    })));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .default_header("x-data-scope", "key-metrics")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    client.key_metrics(&symbol).await.unwrap();
    client
        .key_metrics(KeyMetricsQuery::new(symbol.clone()).with_limit(Limit(0)))
        .await
        .unwrap();

    let periods = [
        StatementPeriod::from(FiscalPeriod::Q1),
        StatementPeriod::from(FiscalPeriod::Q2),
        StatementPeriod::from(FiscalPeriod::Q3),
        StatementPeriod::from(FiscalPeriod::Q4),
        StatementPeriod::from(FiscalPeriod::FullYear),
        StatementPeriod::from(RetrievalFrequency::Annual),
        StatementPeriod::from(RetrievalFrequency::Quarterly),
    ];
    for period in periods {
        client
            .key_metrics(
                KeyMetricsQuery::new(symbol.clone())
                    .with_limit(Limit(5))
                    .with_period(period),
            )
            .await
            .unwrap();
    }
    client.key_metrics_ttm(&symbol).await.unwrap();

    let requests = executor.requests();
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "key-metrics"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/key-metrics?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/stable/key-metrics?symbol=BRK.B+%2F+Class+A&limit=0",
            "https://proxy.example/router/stable/key-metrics?symbol=BRK.B+%2F+Class+A&limit=5&period=Q1",
            "https://proxy.example/router/stable/key-metrics?symbol=BRK.B+%2F+Class+A&limit=5&period=Q2",
            "https://proxy.example/router/stable/key-metrics?symbol=BRK.B+%2F+Class+A&limit=5&period=Q3",
            "https://proxy.example/router/stable/key-metrics?symbol=BRK.B+%2F+Class+A&limit=5&period=Q4",
            "https://proxy.example/router/stable/key-metrics?symbol=BRK.B+%2F+Class+A&limit=5&period=FY",
            "https://proxy.example/router/stable/key-metrics?symbol=BRK.B+%2F+Class+A&limit=5&period=annual",
            "https://proxy.example/router/stable/key-metrics?symbol=BRK.B+%2F+Class+A&limit=5&period=quarter",
            "https://proxy.example/router/stable/key-metrics-ttm?symbol=BRK.B+%2F+Class+A",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_use_the_same_typed_contracts() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/key-metrics?symbol=AAPL",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/key-metrics-ttm?symbol=AAPL&apikey=query-secret",
            None,
        ),
    ] {
        let response = if expected_header.is_some() {
            HISTORICAL
        } else {
            TTM
        };
        let executor = Arc::new(FixtureExecutor::new([json_fixture(response)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        if expected_header.is_some() {
            let rows = client
                .key_metrics(Ticker::new("AAPL").unwrap())
                .await
                .unwrap();
            assert_eq!(rows[0].working_capital, -17_674_000_000);
        } else {
            let rows = client
                .key_metrics_ttm(Ticker::new("AAPL").unwrap())
                .await
                .unwrap();
            assert_eq!(rows[0].market_cap, 4_874_072_686_740);
        }

        let requests = executor.requests();
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some((name, value)) => assert_eq!(requests[0].expose_headers()[name], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}
