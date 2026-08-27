mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        statements::{
            BalanceSheetStatementQuery, BalanceSheetStatementTtmQuery, balance_sheet_statement,
            balance_sheet_statement_ttm,
        },
    },
    query::{FiscalPeriod, RetrievalFrequency},
    transport::HttpMethod,
    types::{Limit, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const HISTORICAL: &[u8] = include_bytes!("fixtures/balance_sheet_statement.json");
const TTM: &[u8] = include_bytes!("fixtures/balance_sheet_statement_ttm.json");

#[test]
fn descriptors_use_exact_paths_distinct_response_shapes_and_documented_metadata() {
    let symbol = Ticker::new("AAPL").unwrap();
    let historical_query = BalanceSheetStatementQuery::new(symbol.clone())
        .with_limit(Limit(1_000))
        .with_period(RetrievalFrequency::Annual);
    let ttm_query = BalanceSheetStatementTtmQuery::new(symbol.clone()).with_limit(Limit(1_000));

    assert_eq!(historical_query.symbol(), &symbol);
    assert_eq!(historical_query.limit(), Some(Limit(1_000)));
    assert_eq!(
        historical_query.period(),
        Some(RetrievalFrequency::Annual.into())
    );
    assert_eq!(ttm_query.symbol(), &symbol);
    assert_eq!(ttm_query.limit(), Some(Limit(1_000)));

    assert_facts(
        &balance_sheet_statement(historical_query),
        "balance-sheet-statement",
    );
    assert_facts(
        &balance_sheet_statement_ttm(ttm_query),
        "balance-sheet-statement-ttm",
    );
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Worldwide
    );
    assert_eq!(endpoint.metadata().access(), AccessRequirement::Unspecified);
    assert_eq!(
        endpoint.metadata().bounds(),
        EndpointBounds::new().with_response_rows(1_000)
    );
    assert_eq!(endpoint.metadata().conditional_plan(), None);
    assert_eq!(endpoint.metadata().realtime(), None);
}

#[tokio::test]
async fn proxy_client_uses_exact_queries_custom_auth_and_decodes_both_bare_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(HISTORICAL),
        json_fixture(TTM),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    let historical = client
        .balance_sheet_statement(
            BalanceSheetStatementQuery::new(symbol.clone())
                .with_limit(Limit(5))
                .with_period(FiscalPeriod::Q1),
        )
        .await
        .unwrap();
    let ttm = client
        .balance_sheet_statement_ttm(
            BalanceSheetStatementTtmQuery::new(symbol).with_limit(Limit(7)),
        )
        .await
        .unwrap();

    assert_eq!(historical.len(), 1);
    assert_eq!(historical[0].total_assets, 359_241_000_000);
    assert_eq!(historical[0].retained_earnings, -14_264_000_000);
    assert_eq!(ttm.len(), 1);
    assert_eq!(ttm[0].total_assets, 371_082_000_000);

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(
        requests
            .iter()
            .all(|request| { request.expose_headers()["x-router-token"] == "Token proxy-secret" })
    );
    let urls = requests
        .iter()
        .map(|request| request.expose_url().as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        urls,
        [
            "https://proxy.example/router/stable/balance-sheet-statement?symbol=BRK.B+%2F+Class+A&limit=5&period=Q1",
            "https://proxy.example/router/stable/balance-sheet-statement-ttm?symbol=BRK.B+%2F+Class+A&limit=7",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_use_the_same_typed_contracts() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/balance-sheet-statement?symbol=AAPL",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/balance-sheet-statement-ttm?symbol=AAPL&apikey=query-secret",
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
            client
                .balance_sheet_statement(Ticker::new("AAPL").unwrap())
                .await
                .unwrap();
        } else {
            client
                .balance_sheet_statement_ttm(Ticker::new("AAPL").unwrap())
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
