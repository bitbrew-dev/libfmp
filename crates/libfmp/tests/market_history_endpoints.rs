mod support;

use std::{str::FromStr, sync::Arc};

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        market::{
            HistoricalIndustryPeQuery, HistoricalIndustryPerformanceQuery, HistoricalSectorPeQuery,
            HistoricalSectorPerformanceQuery, historical_industry_pe,
            historical_industry_performance, historical_sector_pe, historical_sector_performance,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::market::{IndustryPe, IndustryPerformance, SectorPe, SectorPerformance},
    transport::HttpMethod,
    types::{Date, ExchangeCode, Industry, Sector},
};

use support::{FixtureExecutor, json_fixture};

const SECTOR_PERFORMANCE: &[u8] = include_bytes!("fixtures/historical_sector_performance.json");
const INDUSTRY_PERFORMANCE: &[u8] = include_bytes!("fixtures/historical_industry_performance.json");
const SECTOR_PE: &[u8] = include_bytes!("fixtures/historical_sector_pe.json");
const INDUSTRY_PE: &[u8] = include_bytes!("fixtures/historical_industry_pe.json");

fn from() -> Date {
    Date::from_str("2024-02-01").unwrap()
}

fn to() -> Date {
    Date::from_str("2024-03-01").unwrap()
}

fn sector() -> Sector {
    Sector::new("Future & Energy / Utilities").unwrap()
}

fn industry() -> Industry {
    Industry::new("Research & Consulting / Services").unwrap()
}

#[test]
fn descriptors_use_exact_paths_bare_rows_and_only_documented_metadata() {
    let sector_performance =
        historical_sector_performance(HistoricalSectorPerformanceQuery::new(sector()));
    let industry_performance =
        historical_industry_performance(HistoricalIndustryPerformanceQuery::new(industry()));
    let sector_pe = historical_sector_pe(HistoricalSectorPeQuery::new(sector()));
    let industry_pe = historical_industry_pe(HistoricalIndustryPeQuery::new(industry()));

    assert_facts(&sector_performance, "historical-sector-performance");
    assert_facts(&industry_performance, "historical-industry-performance");
    assert_facts(&sector_pe, "historical-sector-pe");
    assert_facts(&industry_pe, "historical-industry-pe");
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
    _: &EndpointSpec<HistoricalSectorPerformanceQuery, Vec<SectorPerformance>>,
    _: &EndpointSpec<HistoricalIndustryPerformanceQuery, Vec<IndustryPerformance>>,
    _: &EndpointSpec<HistoricalSectorPeQuery, Vec<SectorPe>>,
    _: &EndpointSpec<HistoricalIndustryPeQuery, Vec<IndustryPe>>,
) {
}

#[tokio::test]
async fn proxy_preserves_every_optional_combination_exact_order_encoding_and_headers() {
    let responses = [
        SECTOR_PERFORMANCE,
        SECTOR_PE,
        SECTOR_PERFORMANCE,
        SECTOR_PE,
        SECTOR_PERFORMANCE,
        SECTOR_PE,
        SECTOR_PERFORMANCE,
        SECTOR_PE,
        INDUSTRY_PERFORMANCE,
        INDUSTRY_PE,
        INDUSTRY_PERFORMANCE,
        INDUSTRY_PE,
        INDUSTRY_PERFORMANCE,
        INDUSTRY_PE,
        INDUSTRY_PERFORMANCE,
        INDUSTRY_PE,
    ];
    let executor = Arc::new(FixtureExecutor::new(
        responses.into_iter().map(json_fixture),
    ));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "market history")
        .executor(executor.clone())
        .build()
        .unwrap();
    let exchange = ExchangeCode::new("NEW / EXCHANGE").unwrap();

    client
        .historical_sector_performance(HistoricalSectorPerformanceQuery::new(sector()))
        .await
        .unwrap();
    client
        .historical_sector_pe(HistoricalSectorPeQuery::new(sector()).with_from(from()))
        .await
        .unwrap();
    client
        .historical_sector_performance(
            HistoricalSectorPerformanceQuery::new(sector()).with_exchange(exchange.clone()),
        )
        .await
        .unwrap();
    client
        .historical_sector_pe(HistoricalSectorPeQuery::new(sector()).with_to(to()))
        .await
        .unwrap();
    client
        .historical_sector_performance(
            HistoricalSectorPerformanceQuery::new(sector())
                .with_from(from())
                .with_exchange(exchange.clone()),
        )
        .await
        .unwrap();
    client
        .historical_sector_pe(
            HistoricalSectorPeQuery::new(sector())
                .with_from(from())
                .with_to(to()),
        )
        .await
        .unwrap();
    client
        .historical_sector_performance(
            HistoricalSectorPerformanceQuery::new(sector())
                .with_exchange(exchange.clone())
                .with_to(to()),
        )
        .await
        .unwrap();
    let sector_rows = client
        .historical_sector_pe(
            HistoricalSectorPeQuery::new(sector())
                .with_from(from())
                .with_exchange(exchange.clone())
                .with_to(to()),
        )
        .await
        .unwrap();

    client
        .historical_industry_performance(HistoricalIndustryPerformanceQuery::new(industry()))
        .await
        .unwrap();
    client
        .historical_industry_pe(HistoricalIndustryPeQuery::new(industry()).with_from(from()))
        .await
        .unwrap();
    client
        .historical_industry_performance(
            HistoricalIndustryPerformanceQuery::new(industry()).with_exchange(exchange.clone()),
        )
        .await
        .unwrap();
    client
        .historical_industry_pe(HistoricalIndustryPeQuery::new(industry()).with_to(to()))
        .await
        .unwrap();
    client
        .historical_industry_performance(
            HistoricalIndustryPerformanceQuery::new(industry())
                .with_exchange(exchange.clone())
                .with_from(from()),
        )
        .await
        .unwrap();
    client
        .historical_industry_pe(
            HistoricalIndustryPeQuery::new(industry())
                .with_from(from())
                .with_to(to()),
        )
        .await
        .unwrap();
    client
        .historical_industry_performance(
            HistoricalIndustryPerformanceQuery::new(industry())
                .with_exchange(exchange.clone())
                .with_to(to()),
        )
        .await
        .unwrap();
    let industry_rows = client
        .historical_industry_pe(
            HistoricalIndustryPeQuery::new(industry())
                .with_exchange(exchange)
                .with_from(from())
                .with_to(to()),
        )
        .await
        .unwrap();

    assert_eq!(sector_rows[0].sector.as_str(), "Energy");
    assert_eq!(sector_rows[0].date, to());
    assert!((sector_rows[0].pe - 5.4165892628211205).abs() < 1e-14);
    assert_eq!(industry_rows[0].industry.as_str(), "Biotechnology");
    assert_eq!(industry_rows[0].date, to());
    assert!((industry_rows[0].pe - 8.129037884885042).abs() < 1e-14);

    let requests = executor.requests();
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "market history"
    }));
    let urls = requests
        .iter()
        .map(|request| request.expose_url().as_str())
        .collect::<Vec<_>>();
    let root = "https://proxy.example/router/gateway/stable";
    let sector = "sector=Future+%26+Energy+%2F+Utilities";
    let industry = "industry=Research+%26+Consulting+%2F+Services";
    assert_eq!(
        urls,
        [
            format!("{root}/historical-sector-performance?{sector}"),
            format!("{root}/historical-sector-pe?from=2024-02-01&{sector}"),
            format!("{root}/historical-sector-performance?exchange=NEW+%2F+EXCHANGE&{sector}"),
            format!("{root}/historical-sector-pe?{sector}&to=2024-03-01"),
            format!(
                "{root}/historical-sector-performance?from=2024-02-01&exchange=NEW+%2F+EXCHANGE&{sector}"
            ),
            format!("{root}/historical-sector-pe?from=2024-02-01&{sector}&to=2024-03-01"),
            format!(
                "{root}/historical-sector-performance?exchange=NEW+%2F+EXCHANGE&{sector}&to=2024-03-01"
            ),
            format!(
                "{root}/historical-sector-pe?from=2024-02-01&exchange=NEW+%2F+EXCHANGE&{sector}&to=2024-03-01"
            ),
            format!("{root}/historical-industry-performance?{industry}"),
            format!("{root}/historical-industry-pe?{industry}&from=2024-02-01"),
            format!("{root}/historical-industry-performance?{industry}&exchange=NEW+%2F+EXCHANGE"),
            format!("{root}/historical-industry-pe?{industry}&to=2024-03-01"),
            format!(
                "{root}/historical-industry-performance?{industry}&exchange=NEW+%2F+EXCHANGE&from=2024-02-01"
            ),
            format!("{root}/historical-industry-pe?{industry}&from=2024-02-01&to=2024-03-01"),
            format!(
                "{root}/historical-industry-performance?{industry}&exchange=NEW+%2F+EXCHANGE&to=2024-03-01"
            ),
            format!(
                "{root}/historical-industry-pe?{industry}&exchange=NEW+%2F+EXCHANGE&from=2024-02-01&to=2024-03-01"
            ),
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_only_required_filters() {
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
            .historical_sector_performance(sector())
            .await
            .unwrap();
        client
            .historical_industry_performance(industry())
            .await
            .unwrap();
        client.historical_sector_pe(sector()).await.unwrap();
        client.historical_industry_pe(industry()).await.unwrap();

        let requests = executor.requests();
        assert_eq!(
            requests
                .iter()
                .map(|request| request.expose_url().as_str())
                .collect::<Vec<_>>(),
            [
                format!(
                    "https://financialmodelingprep.com/stable/historical-sector-performance?sector=Future+%26+Energy+%2F+Utilities{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/historical-industry-performance?industry=Research+%26+Consulting+%2F+Services{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/historical-sector-pe?sector=Future+%26+Energy+%2F+Utilities{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/historical-industry-pe?industry=Research+%26+Consulting+%2F+Services{suffix}"
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
            .historical_sector_performance(sector())
            .await
            .unwrap_err(),
        client
            .historical_industry_performance(industry())
            .await
            .unwrap_err(),
        client.historical_sector_pe(sector()).await.unwrap_err(),
        client.historical_industry_pe(industry()).await.unwrap_err(),
    ];

    for (error, endpoint) in errors.iter().zip([
        "historical-sector-performance",
        "historical-industry-performance",
        "historical-sector-pe",
        "historical-industry-pe",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}
