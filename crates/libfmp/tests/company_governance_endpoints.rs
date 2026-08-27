mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        company::{
            ExecutiveCompensationBenchmarkQuery, ExecutiveCompensationQuery, KeyExecutivesQuery,
            executive_compensation, executive_compensation_benchmark, key_executives,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    transport::HttpMethod,
    types::{BenchmarkYear, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const EXECUTIVES: &[u8] = include_bytes!("fixtures/company_key_executives.json");
const EXECUTIVES_DYNAMIC: &[u8] = include_bytes!("fixtures/company_key_executives_dynamic.json");
const COMPENSATION: &[u8] = include_bytes!("fixtures/company_executive_compensation.json");
const COMPENSATION_LARGE: &[u8] =
    include_bytes!("fixtures/company_executive_compensation_large.json");
const BENCHMARK: &[u8] = include_bytes!("fixtures/company_executive_compensation_benchmark.json");
const EMPTY: &[u8] = include_bytes!("fixtures/company_empty.json");

#[test]
fn descriptors_use_exact_paths_queries_geography_and_no_undocumented_metadata() {
    let symbol = Ticker::new("AAPL").unwrap();
    let year = BenchmarkYear::new("FY 2024/25").unwrap();
    let executives_query = KeyExecutivesQuery::new(symbol.clone());
    let compensation_query = ExecutiveCompensationQuery::new(symbol.clone());
    let benchmark_without_year = ExecutiveCompensationBenchmarkQuery::new();
    let benchmark_with_year = ExecutiveCompensationBenchmarkQuery::new().with_year(year.clone());

    assert_eq!(executives_query.symbol(), &symbol);
    assert_eq!(compensation_query.symbol(), &symbol);
    assert_eq!(benchmark_without_year.year(), None);
    assert_eq!(benchmark_with_year.year(), Some(&year));

    assert_facts(
        &key_executives(executives_query),
        "key-executives",
        GeographicAvailability::Worldwide,
    );
    assert_facts(
        &executive_compensation(compensation_query),
        "governance-executive-compensation",
        GeographicAvailability::UsOnly,
    );
    assert_facts(
        &executive_compensation_benchmark(benchmark_without_year),
        "executive-compensation-benchmark",
        GeographicAvailability::UsOnly,
    );
}

fn assert_facts<Q, R>(
    endpoint: &EndpointSpec<Q, R>,
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

#[tokio::test]
async fn proxy_client_preserves_required_symbols_and_omitted_or_present_string_year() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(EXECUTIVES),
        json_fixture(COMPENSATION),
        json_fixture(BENCHMARK),
        json_fixture(BENCHMARK),
    ]));
    let client = proxy_client(executor.clone());

    let executives = client
        .key_executives(Ticker::new("BRK.B").unwrap())
        .await
        .unwrap();
    assert_eq!(executives[0].currency_pay.as_str(), "USD");

    let compensation = client
        .executive_compensation(Ticker::new("AAPL").unwrap())
        .await
        .unwrap();
    assert_eq!(compensation[0].year, 2025);

    let benchmark = client
        .executive_compensation_benchmark(ExecutiveCompensationBenchmarkQuery::new())
        .await
        .unwrap();
    assert_eq!(benchmark[0].year, 2024);

    client
        .executive_compensation_benchmark(
            ExecutiveCompensationBenchmarkQuery::new()
                .with_year(BenchmarkYear::new("FY 2024/25").unwrap()),
        )
        .await
        .unwrap();

    let requests = executor.requests();
    let urls = requests
        .iter()
        .map(|request| request.expose_url().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        urls,
        [
            "https://proxy.example/router/stable/key-executives?symbol=BRK.B",
            "https://proxy.example/router/stable/governance-executive-compensation?symbol=AAPL",
            "https://proxy.example/router/stable/executive-compensation-benchmark",
            "https://proxy.example/router/stable/executive-compensation-benchmark?year=FY+2024%2F25",
        ]
    );
    assert!(!urls[2].contains('?'));
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
}

#[tokio::test]
async fn governance_array_contracts_preserve_multiple_dynamic_large_and_empty_payloads() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(EXECUTIVES_DYNAMIC),
        json_fixture(COMPENSATION_LARGE),
        json_fixture(EMPTY),
    ]));
    let client = proxy_client(executor);

    let executives = client
        .key_executives(Ticker::new("AAPL").unwrap())
        .await
        .unwrap();
    assert_eq!(executives.len(), 2);
    assert_eq!(executives[1].year_born, Some(serde_json::json!("01980")));

    let compensation = client
        .executive_compensation(Ticker::new("AAPL").unwrap())
        .await
        .unwrap();
    assert_eq!(compensation.len(), 2);
    assert_eq!(compensation[1].total, u64::MAX);

    assert!(
        client
            .executive_compensation_benchmark(ExecutiveCompensationBenchmarkQuery::new())
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_use_the_same_governance_contracts() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/key-executives?symbol=AAPL",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/executive-compensation-benchmark?year=2024&apikey=query-secret",
            None,
        ),
    ] {
        let response = if expected_header.is_some() {
            EXECUTIVES
        } else {
            BENCHMARK
        };
        let executor = Arc::new(FixtureExecutor::new([json_fixture(response)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        if expected_header.is_some() {
            client
                .key_executives(Ticker::new("AAPL").unwrap())
                .await
                .unwrap();
        } else {
            client
                .executive_compensation_benchmark(
                    ExecutiveCompensationBenchmarkQuery::new()
                        .with_year(BenchmarkYear::new("2024").unwrap()),
                )
                .await
                .unwrap();
        }

        let requests = executor.requests();
        assert_eq!(requests[0].expose_url().as_str(), expected_url);
        match expected_header {
            Some((name, value)) => assert_eq!(requests[0].expose_headers()[name], value),
            None => assert!(!requests[0].expose_headers().contains_key("apikey")),
        }
    }
}

fn proxy_client(executor: Arc<FixtureExecutor>) -> Client {
    Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::None)
        .executor(executor)
        .build()
        .unwrap()
}
