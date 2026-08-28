mod support;

use std::sync::Arc;

use libfmp::{
    Client,
    config::Authentication,
    endpoints::{
        EndpointSpec,
        metadata::{AccessRequirement, EndpointBounds, GeographicAvailability},
        statements::{
            BalanceSheetStatementAsReportedQuery, IncomeStatementAsReportedQuery,
            balance_sheet_statement_as_reported, income_statement_as_reported,
        },
    },
    query::RetrievalFrequency,
    transport::HttpMethod,
    types::{Limit, Ticker},
};

use support::{FixtureExecutor, json_fixture};

const INCOME: &[u8] = include_bytes!("fixtures/income_statement_as_reported.json");
const BALANCE: &[u8] = include_bytes!("fixtures/balance_sheet_statement_as_reported.json");

#[test]
fn descriptors_use_exact_paths_shared_bare_row_and_only_documented_response_bound() {
    let symbol = Ticker::new("AAPL").unwrap();
    let income_query = IncomeStatementAsReportedQuery::new(symbol.clone())
        .with_limit(Limit(1_000))
        .with_period(RetrievalFrequency::Annual);
    let balance_query = BalanceSheetStatementAsReportedQuery::new(symbol.clone())
        .with_limit(Limit(1_000))
        .with_period(RetrievalFrequency::Quarterly);

    assert_eq!(income_query.symbol(), &symbol);
    assert_eq!(income_query.limit(), Some(Limit(1_000)));
    assert_eq!(income_query.period(), Some(RetrievalFrequency::Annual));
    assert_eq!(balance_query.symbol(), &symbol);
    assert_eq!(balance_query.limit(), Some(Limit(1_000)));
    assert_eq!(balance_query.period(), Some(RetrievalFrequency::Quarterly));

    assert_facts(
        &income_statement_as_reported(income_query),
        "income-statement-as-reported",
    );
    assert_facts(
        &balance_sheet_statement_as_reported(balance_query),
        "balance-sheet-statement-as-reported",
    );
}

fn assert_facts<Q, R>(endpoint: &EndpointSpec<Q, R>, path: &'static str) {
    assert_eq!(endpoint.method(), HttpMethod::Get);
    assert_eq!(endpoint.id(), path);
    assert_eq!(endpoint.relative_path(), path);
    assert_eq!(
        endpoint.metadata().geography(),
        GeographicAvailability::Unspecified
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
async fn custom_proxy_preserves_exact_queries_auth_default_headers_and_dynamic_arrays() {
    let executor = Arc::new(FixtureExecutor::new([
        json_fixture(INCOME),
        json_fixture(BALANCE),
    ]));
    let client = Client::builder()
        .base_url("https://proxy.example/router")
        .path_prefix("stable")
        .authentication(Authentication::custom_header(
            "x-router-token",
            Some("Token ".to_owned()),
            "proxy-secret",
        ))
        .default_header("x-data-scope", "as-reported")
        .executor(executor.clone())
        .build()
        .unwrap();
    let symbol = Ticker::new("BRK.B / Class A").unwrap();

    let income = client
        .income_statement_as_reported(
            IncomeStatementAsReportedQuery::new(symbol.clone())
                .with_limit(Limit(5))
                .with_period(RetrievalFrequency::Annual),
        )
        .await
        .unwrap();
    let balance = client
        .balance_sheet_statement_as_reported(
            BalanceSheetStatementAsReportedQuery::new(symbol)
                .with_limit(Limit(7))
                .with_period(RetrievalFrequency::Quarterly),
        )
        .await
        .unwrap();

    assert_eq!(income.len(), 1);
    assert_eq!(income[0].data.len(), 24);
    assert_eq!(balance.len(), 1);
    assert_eq!(balance[0].data.len(), 31);

    let requests = executor.requests();
    assert!(
        requests
            .iter()
            .all(|request| request.method() == HttpMethod::Get)
    );
    assert!(requests.iter().all(|request| {
        request.expose_headers()["x-router-token"] == "Token proxy-secret"
            && request.expose_headers()["x-data-scope"] == "as-reported"
    }));
    assert_eq!(
        requests
            .iter()
            .map(|request| request.expose_url().as_str())
            .collect::<Vec<_>>(),
        [
            "https://proxy.example/router/stable/income-statement-as-reported?symbol=BRK.B+%2F+Class+A&limit=5&period=annual",
            "https://proxy.example/router/stable/balance-sheet-statement-as-reported?symbol=BRK.B+%2F+Class+A&limit=7&period=quarter",
        ]
    );
}

#[tokio::test]
async fn direct_header_and_query_auth_use_the_same_as_reported_contract() {
    for (authentication, expected_url, expected_header) in [
        (
            Authentication::fmp_header("header-secret"),
            "https://financialmodelingprep.com/stable/income-statement-as-reported?symbol=AAPL",
            Some(("apikey", "header-secret")),
        ),
        (
            Authentication::fmp_query("query-secret"),
            "https://financialmodelingprep.com/stable/balance-sheet-statement-as-reported?symbol=AAPL&apikey=query-secret",
            None,
        ),
    ] {
        let response = if expected_header.is_some() {
            INCOME
        } else {
            BALANCE
        };
        let executor = Arc::new(FixtureExecutor::new([json_fixture(response)]));
        let client = Client::builder()
            .authentication(authentication)
            .executor(executor.clone())
            .build()
            .unwrap();

        if expected_header.is_some() {
            client
                .income_statement_as_reported(Ticker::new("AAPL").unwrap())
                .await
                .unwrap();
        } else {
            client
                .balance_sheet_statement_as_reported(Ticker::new("AAPL").unwrap())
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
