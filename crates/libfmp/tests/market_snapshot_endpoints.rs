mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        market::{
            IndustryPeSnapshotQuery, IndustryPerformanceSnapshotQuery, SectorPeSnapshotQuery,
            SectorPerformanceSnapshotQuery, industry_pe_snapshot, industry_performance_snapshot,
            sector_pe_snapshot, sector_performance_snapshot,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::market::{IndustryPe, IndustryPerformance, SectorPe, SectorPerformance},
    transport::HttpMethod,
    types::{Date, ExchangeCode, Industry, Sector},
};

use support::{FixtureExecutor, json_fixture};

const SECTOR_PERFORMANCE: &[u8] = include_bytes!("fixtures/sector_performance_snapshot.json");
const INDUSTRY_PERFORMANCE: &[u8] = include_bytes!("fixtures/industry_performance_snapshot.json");
const SECTOR_PE: &[u8] = include_bytes!("fixtures/sector_pe_snapshot.json");
const INDUSTRY_PE: &[u8] = include_bytes!("fixtures/industry_pe_snapshot.json");

fn date() -> Date {
    Date::from_str("2024-02-01").unwrap()
}

#[test]
fn descriptors_use_exact_paths_bare_rows_and_only_documented_metadata() {
    let sector_performance =
        sector_performance_snapshot(SectorPerformanceSnapshotQuery::new(date()));
    let industry_performance =
        industry_performance_snapshot(IndustryPerformanceSnapshotQuery::new(date()));
    let sector_pe = sector_pe_snapshot(SectorPeSnapshotQuery::new(date()));
    let industry_pe = industry_pe_snapshot(IndustryPeSnapshotQuery::new(date()));

    assert_facts(&sector_performance, "sector-performance-snapshot");
    assert_facts(&industry_performance, "industry-performance-snapshot");
    assert_facts(&sector_pe, "sector-pe-snapshot");
    assert_facts(&industry_pe, "industry-pe-snapshot");
    assert_response_types(
        &sector_performance,
        &industry_performance,
        &sector_pe,
        &industry_pe,
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
    _: &EndpointSpec<SectorPerformanceSnapshotQuery, Vec<SectorPerformance>>,
    _: &EndpointSpec<IndustryPerformanceSnapshotQuery, Vec<IndustryPerformance>>,
    _: &EndpointSpec<SectorPeSnapshotQuery, Vec<SectorPe>>,
    _: &EndpointSpec<IndustryPeSnapshotQuery, Vec<IndustryPe>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_exact_paths_query_order_encoding_headers_and_fixtures() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(SECTOR_PERFORMANCE),
        json_fixture(INDUSTRY_PERFORMANCE),
        json_fixture(SECTOR_PE),
        json_fixture(INDUSTRY_PE),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "market snapshots")
        .executor(executor.clone())
        .build()
        .unwrap();
    let exchange = ExchangeCode::new("NEW / EXCHANGE").unwrap();
    let sector = Sector::new("Future & Energy").unwrap();
    let industry = Industry::new("Research & Consulting / Services").unwrap();

    let sector_performance = client
        .sector_performance_snapshot(
            SectorPerformanceSnapshotQuery::new(date())
                .with_exchange(exchange.clone())
                .with_sector(sector.clone()),
        )
        .await
        .unwrap();
    let industry_performance = client
        .industry_performance_snapshot(
            IndustryPerformanceSnapshotQuery::new(date())
                .with_exchange(exchange.clone())
                .with_industry(industry.clone()),
        )
        .await
        .unwrap();
    let sector_pe = client
        .sector_pe_snapshot(
            SectorPeSnapshotQuery::new(date())
                .with_exchange(exchange.clone())
                .with_sector(sector),
        )
        .await
        .unwrap();
    let industry_pe = client
        .industry_pe_snapshot(
            IndustryPeSnapshotQuery::new(date())
                .with_exchange(exchange)
                .with_industry(industry),
        )
        .await
        .unwrap();

    assert_eq!(sector_performance.len(), 1);
    assert_eq!(sector_performance[0].sector.as_str(), "Basic Materials");
    assert!((sector_performance[0].average_change - (-0.31481377464310634)).abs() < f64::EPSILON);
    assert_eq!(industry_performance.len(), 1);
    assert_eq!(
        industry_performance[0].industry.as_str(),
        "Advertising Agencies"
    );
    assert!((industry_performance[0].average_change - 3.8660194344955996).abs() < 1e-14);
    assert_eq!(sector_pe.len(), 1);
    assert!((sector_pe[0].pe - 15.687711758428254).abs() < 1e-14);
    assert_eq!(industry_pe.len(), 1);
    assert!((industry_pe[0].pe - 71.09601665201151).abs() < 1e-13);

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "market snapshots"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/sector-performance-snapshot?date=2024-02-01&exchange=NEW+%2F+EXCHANGE&sector=Future+%26+Energy",
            "https://proxy.example/router/gateway/stable/industry-performance-snapshot?date=2024-02-01&exchange=NEW+%2F+EXCHANGE&industry=Research+%26+Consulting+%2F+Services",
            "https://proxy.example/router/gateway/stable/sector-pe-snapshot?date=2024-02-01&exchange=NEW+%2F+EXCHANGE&sector=Future+%26+Energy",
            "https://proxy.example/router/gateway/stable/industry-pe-snapshot?date=2024-02-01&exchange=NEW+%2F+EXCHANGE&industry=Research+%26+Consulting+%2F+Services",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_omitted_optional_filters() {
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
            json_fixture(SECTOR_PERFORMANCE),
            json_fixture(INDUSTRY_PERFORMANCE),
            json_fixture(SECTOR_PE),
            json_fixture(INDUSTRY_PE),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .sector_performance_snapshot(SectorPerformanceSnapshotQuery::new(date()))
            .await
            .unwrap();
        client
            .industry_performance_snapshot(IndustryPerformanceSnapshotQuery::new(date()))
            .await
            .unwrap();
        client
            .sector_pe_snapshot(SectorPeSnapshotQuery::new(date()))
            .await
            .unwrap();
        client
            .industry_pe_snapshot(IndustryPeSnapshotQuery::new(date()))
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
                    "https://financialmodelingprep.com/stable/sector-performance-snapshot?date=2024-02-01{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/industry-performance-snapshot?date=2024-02-01{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/sector-pe-snapshot?date=2024-02-01{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/industry-pe-snapshot?date=2024-02-01{suffix}"
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
async fn exchange_and_dimension_filters_remain_independently_optional() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(SECTOR_PERFORMANCE),
        json_fixture(INDUSTRY_PERFORMANCE),
        json_fixture(SECTOR_PE),
        json_fixture(INDUSTRY_PE),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor.clone())
        .build()
        .unwrap();

    client
        .sector_performance_snapshot(
            SectorPerformanceSnapshotQuery::new(date())
                .with_exchange(ExchangeCode::new("NASDAQ").unwrap()),
        )
        .await
        .unwrap();
    client
        .industry_performance_snapshot(
            IndustryPerformanceSnapshotQuery::new(date())
                .with_industry(Industry::new("Biotechnology").unwrap()),
        )
        .await
        .unwrap();
    client
        .sector_pe_snapshot(
            SectorPeSnapshotQuery::new(date()).with_sector(Sector::new("Energy").unwrap()),
        )
        .await
        .unwrap();
    client
        .industry_pe_snapshot(
            IndustryPeSnapshotQuery::new(date()).with_exchange(ExchangeCode::new("NYSE").unwrap()),
        )
        .await
        .unwrap();

    assert_eq!(
        executor
            .requests()
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://financialmodelingprep.com/stable/sector-performance-snapshot?date=2024-02-01&exchange=NASDAQ",
            "https://financialmodelingprep.com/stable/industry-performance-snapshot?date=2024-02-01&industry=Biotechnology",
            "https://financialmodelingprep.com/stable/sector-pe-snapshot?date=2024-02-01&sector=Energy",
            "https://financialmodelingprep.com/stable/industry-pe-snapshot?date=2024-02-01&exchange=NYSE",
        ]
    );
}

#[tokio::test]
async fn malformed_non_array_responses_keep_each_endpoint_identity() {
    let executor = Arc::new(FixtureExecutor::new(
        std::iter::repeat_with(|| json_fixture(b"{}")).take(4),
    ));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();

    let errors = [
        client
            .sector_performance_snapshot(SectorPerformanceSnapshotQuery::new(date()))
            .await
            .unwrap_err(),
        client
            .industry_performance_snapshot(IndustryPerformanceSnapshotQuery::new(date()))
            .await
            .unwrap_err(),
        client
            .sector_pe_snapshot(SectorPeSnapshotQuery::new(date()))
            .await
            .unwrap_err(),
        client
            .industry_pe_snapshot(IndustryPeSnapshotQuery::new(date()))
            .await
            .unwrap_err(),
    ];

    for (error, endpoint) in errors.iter().zip([
        "sector-performance-snapshot",
        "industry-performance-snapshot",
        "sector-pe-snapshot",
        "industry-pe-snapshot",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}
