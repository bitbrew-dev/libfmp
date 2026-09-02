mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        esg::{EsgBenchmarkQuery, EsgSymbolQuery, esg_benchmark, esg_disclosures, esg_ratings},
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    transport::HttpMethod,
    types::{BenchmarkYear, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const DISCLOSURES: &[u8] = include_bytes!("fixtures/esg_disclosures.json");
const RATINGS: &[u8] = include_bytes!("fixtures/esg_ratings.json");
const BENCHMARK: &[u8] = include_bytes!("fixtures/esg_benchmark.json");

#[test]
fn descriptors_have_exact_identity_paths_queries_and_only_us_geography() {
    let symbol = Ticker::new("AAPL").unwrap();
    let owned = EsgSymbolQuery::from(symbol.clone());
    let borrowed = EsgSymbolQuery::from(&symbol);
    let benchmark_year = BenchmarkYear::new("2023").unwrap();
    let benchmark_query = EsgBenchmarkQuery::new().with_year(benchmark_year.clone());

    assert_eq!(owned.symbol(), &symbol);
    assert_eq!(borrowed.symbol(), &symbol);
    assert_eq!(benchmark_query.year(), Some(&benchmark_year));

    assert_facts(&esg_disclosures(owned), "esg-disclosures");
    assert_facts(&esg_ratings(borrowed), "esg-ratings");
    assert_facts(&esg_benchmark(benchmark_query), "esg-benchmark");
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
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

#[tokio::test]
async fn proxy_routes_all_methods_with_exact_queries_custom_auth_headers_and_fixtures() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(DISCLOSURES),
        json_fixture(RATINGS),
        json_fixture(BENCHMARK),
        json_fixture(BENCHMARK),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("/gateway/stable/")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "esg")
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    assert_eq!(
        client.esg_disclosures(&symbol).await.unwrap()[0].esg_score,
        56.79
    );
    assert_eq!(
        client.esg_ratings(symbol).await.unwrap()[0].esg_risk_rating,
        "B"
    );
    assert_eq!(
        client
            .esg_benchmark(EsgBenchmarkQuery::new())
            .await
            .unwrap()[0]
            .esg_score,
        65.63
    );
    client
        .esg_benchmark(
            EsgBenchmarkQuery::new().with_year(BenchmarkYear::new("FY 2024/25").unwrap()),
        )
        .await
        .unwrap();

    let requests = executor.requests();
    assert_eq!(requests.len(), 4);
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "esg"
    }));
    assert_eq!(
        request_urls(&requests),
        [
            "https://proxy.example/router/gateway/stable/esg-disclosures?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/gateway/stable/esg-ratings?symbol=BRK.B+%2F+Class+A",
            "https://proxy.example/router/gateway/stable/esg-benchmark",
            "https://proxy.example/router/gateway/stable/esg-benchmark?year=FY+2024%2F25",
        ]
    );
}

#[tokio::test]
async fn direct_fmp_header_and_query_auth_preserve_esg_contracts() {
    for (authentication, expected_url, expected_header, disclosure) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/esg-disclosures?symbol=AAPL",
            Some("header-secret"),
            true,
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/esg-benchmark?year=2023&apikey=query-secret",
            None,
            false,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([json_fixture(if disclosure {
            DISCLOSURES
        } else {
            BENCHMARK
        })]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        if disclosure {
            client
                .esg_disclosures(Ticker::new("AAPL").unwrap())
                .await
                .unwrap();
        } else {
            client
                .esg_benchmark(
                    EsgBenchmarkQuery::new().with_year(BenchmarkYear::new("2023").unwrap()),
                )
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
async fn malformed_non_array_responses_preserve_all_endpoint_identities() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(b"{}"),
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
            .esg_disclosures(Ticker::new("AAPL").unwrap())
            .await
            .unwrap_err(),
        client
            .esg_ratings(Ticker::new("AAPL").unwrap())
            .await
            .unwrap_err(),
        client
            .esg_benchmark(EsgBenchmarkQuery::new())
            .await
            .unwrap_err(),
    ];

    for (error, id) in errors
        .iter()
        .zip(["esg-disclosures", "esg-ratings", "esg-benchmark"])
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
