mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        insider_trading::{
            InsiderReportingNameSearchQuery, InsiderTradeStatisticsQuery, insider_trade_statistics,
            insider_transaction_types, search_insider_reporting_names,
        },
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
    },
    error::ErrorCategory,
    responses::insider_trading::{
        InsiderReportingName, InsiderTradeStatistics, InsiderTransactionType,
    },
    transport::HttpMethod,
    types::{CalendarQuarter, CalendarYear, SearchTerm, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const REPORTING_NAMES: &[u8] = include_bytes!("fixtures/insider_reporting_names.json");
const TRANSACTION_TYPES: &[u8] = include_bytes!("fixtures/insider_transaction_types.json");
const STATISTICS: &[u8] = include_bytes!("fixtures/insider_trade_statistics.json");

#[test]
fn descriptors_use_exact_paths_bare_rows_and_only_documented_metadata() {
    let names = search_insider_reporting_names(InsiderReportingNameSearchQuery::new(
        SearchTerm::new("Zuckerberg").unwrap(),
    ));
    let transaction_types = insider_transaction_types();
    let statistics = insider_trade_statistics(InsiderTradeStatisticsQuery::new(
        Ticker::new("AAPL").unwrap(),
    ));

    assert_facts(&names, "insider-trading/reporting-name");
    assert_facts(&transaction_types, "insider-trading-transaction-type");
    assert_facts(&statistics, "insider-trading/statistics");
    assert_response_types(&names, &transaction_types, &statistics);
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
    _: &EndpointSpec<InsiderReportingNameSearchQuery, Vec<InsiderReportingName>>,
    _: &EndpointSpec<(), Vec<InsiderTransactionType>>,
    _: &EndpointSpec<InsiderTradeStatisticsQuery, Vec<InsiderTradeStatistics>>,
) {
}

#[tokio::test]
async fn custom_proxy_preserves_exact_queries_headers_and_fixture_identities() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(REPORTING_NAMES),
        json_fixture(TRANSACTION_TYPES),
        json_fixture(STATISTICS),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("gateway/stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "insider reference")
        .executor(executor.clone())
        .build()
        .unwrap();

    let names = client
        .search_insider_reporting_names(SearchTerm::new("Zuckerberg, Mark / Meta").unwrap())
        .await
        .unwrap();
    let transaction_types = client.insider_transaction_types().await.unwrap();
    let statistics = client
        .insider_trade_statistics(Ticker::new("BRK.B / Class A").unwrap())
        .await
        .unwrap();

    assert_eq!(names.len(), 1);
    assert_eq!(names[0].reporting_cik.as_str(), "0001548760");
    assert_eq!(names[0].reporting_name, "Zuckerberg Mark");
    assert_eq!(transaction_types.len(), 1);
    assert_eq!(transaction_types[0].transaction_type.as_str(), "A-Award");
    assert_eq!(statistics.len(), 1);
    let row = &statistics[0];
    assert_eq!(row.symbol.as_str(), "AAPL");
    assert_eq!(row.cik.as_str(), "0000320193");
    assert_eq!(row.year, CalendarYear(2026));
    assert_eq!(row.quarter, CalendarQuarter::new(2).unwrap());
    assert_eq!(row.acquired_transactions, 7);
    assert_eq!(row.disposed_transactions, 40);
    assert_eq!(row.acquired_disposed_ratio, 0.175);
    assert_eq!(row.total_acquired, 303_199);
    assert_eq!(row.total_disposed, 927_380);
    assert_eq!(row.average_acquired, 43_314.142_9);
    assert_eq!(row.average_disposed, 23_184.5);
    assert_eq!(row.total_purchases, 0);
    assert_eq!(row.total_sales, 14);

    let requests = executor.requests();
    assert!(requests.iter().all(|request| {
        request.method() == HttpMethod::Get
            && request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "insider reference"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/gateway/stable/insider-trading/reporting-name?name=Zuckerberg%2C+Mark+%2F+Meta",
            "https://proxy.example/router/gateway/stable/insider-trading-transaction-type",
            "https://proxy.example/router/gateway/stable/insider-trading/statistics?symbol=BRK.B+%2F+Class+A",
        ]
    );
    assert!(!requests[1].expose_url().as_str().contains('?'));
}

#[tokio::test]
async fn direct_header_and_query_auth_preserve_exact_urls() {
    for (authentication, suffix, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "",
            Some("header-secret"),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "?apikey=query-secret",
            None,
        ),
    ] {
        let executor = Arc::new(FixtureExecutor::new([
            json_fixture(REPORTING_NAMES),
            json_fixture(TRANSACTION_TYPES),
            json_fixture(STATISTICS),
        ]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        client
            .search_insider_reporting_names(SearchTerm::new("Zuckerberg").unwrap())
            .await
            .unwrap();
        client.insider_transaction_types().await.unwrap();
        client
            .insider_trade_statistics(Ticker::new("AAPL").unwrap())
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
                    "https://financialmodelingprep.com/stable/insider-trading/reporting-name?name=Zuckerberg{}",
                    if suffix.is_empty() {
                        ""
                    } else {
                        "&apikey=query-secret"
                    }
                ),
                format!(
                    "https://financialmodelingprep.com/stable/insider-trading-transaction-type{suffix}"
                ),
                format!(
                    "https://financialmodelingprep.com/stable/insider-trading/statistics?symbol=AAPL{}",
                    if suffix.is_empty() {
                        ""
                    } else {
                        "&apikey=query-secret"
                    }
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
        json_fixture(b"{}"),
    ]));
    let client = Client::builder()
        .authentication(Authentication::fmp_header("secret"))
        .executor(executor)
        .build()
        .unwrap();

    let errors = [
        client
            .search_insider_reporting_names(SearchTerm::new("Zuckerberg").unwrap())
            .await
            .unwrap_err(),
        client.insider_transaction_types().await.unwrap_err(),
        client
            .insider_trade_statistics(Ticker::new("AAPL").unwrap())
            .await
            .unwrap_err(),
    ];

    for (error, endpoint) in errors.iter().zip([
        "insider-trading/reporting-name",
        "insider-trading-transaction-type",
        "insider-trading/statistics",
    ]) {
        assert_eq!(error.category(), ErrorCategory::Decode);
        assert_eq!(error.endpoint(), Some(endpoint));
        assert_eq!(error.status_code(), Some(200));
    }
}
